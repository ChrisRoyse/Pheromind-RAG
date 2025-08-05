// TypeScript Payment Gateway Integration
import { EventEmitter } from 'events';
import axios, { AxiosInstance } from 'axios';
import crypto from 'crypto';
import { v4 as uuidv4 } from 'uuid';

// Payment provider interfaces
interface PaymentProvider {
  processPayment(request: PaymentRequest): Promise<PaymentResult>;
  refundPayment(transactionId: string, amount?: number): Promise<RefundResult>;
  getTransactionStatus(transactionId: string): Promise<TransactionStatus>;
  validateWebhook(payload: any, signature: string): boolean;
}

// Main payment gateway class
export class PaymentGateway extends EventEmitter {
  private providers: Map<string, PaymentProvider>;
  private defaultProvider: string;
  private webhookSecret: string;
  private rateLimiter: RateLimiter;
  private transactionLogger: TransactionLogger;
  private fraudDetector: FraudDetector;

  constructor(config: PaymentGatewayConfig) {
    super();
    this.providers = new Map();
    this.defaultProvider = config.defaultProvider;
    this.webhookSecret = config.webhookSecret;
    this.rateLimiter = new RateLimiter(config.rateLimits);
    this.transactionLogger = new TransactionLogger(config.logPath);
    this.fraudDetector = new FraudDetector(config.fraudRules);

    // Initialize providers
    this.initializeProviders(config.providers);
  }

  private initializeProviders(providerConfigs: ProviderConfig[]): void {
    for (const config of providerConfigs) {
      switch (config.name) {
        case 'stripe':
          this.providers.set('stripe', new StripeProvider(config));
          break;
        case 'paypal':
          this.providers.set('paypal', new PayPalProvider(config));
          break;
        case 'square':
          this.providers.set('square', new SquareProvider(config));
          break;
        default:
          throw new Error(`Unknown payment provider: ${config.name}`);
      }
    }
  }

  async processPayment(request: PaymentRequest): Promise<PaymentResult> {
    const transactionId = uuidv4();
    
    try {
      // Rate limiting
      await this.rateLimiter.checkLimit(request.customerId);

      // Fraud detection
      const fraudCheck = await this.fraudDetector.analyze(request);
      if (fraudCheck.isHighRisk) {
        throw new PaymentError('FRAUD_DETECTED', fraudCheck.reason);
      }

      // Validate request
      this.validatePaymentRequest(request);

      // Select provider
      const provider = this.selectProvider(request);

      // Process payment
      const startTime = Date.now();
      const result = await provider.processPayment({
        ...request,
        transactionId,
        metadata: {
          ...request.metadata,
          gatewayVersion: '2.0',
          processingTime: new Date().toISOString()
        }
      });

      // Log transaction
      await this.transactionLogger.log({
        transactionId,
        provider: request.provider || this.defaultProvider,
        amount: request.amount,
        currency: request.currency,
        status: result.status,
        processingTime: Date.now() - startTime,
        customerId: request.customerId
      });

      // Emit success event
      this.emit('payment:success', {
        transactionId,
        amount: request.amount,
        provider: request.provider || this.defaultProvider
      });

      return {
        ...result,
        transactionId,
        gatewayTransactionId: transactionId
      };

    } catch (error) {
      // Log error
      await this.transactionLogger.logError({
        transactionId,
        error: error.message,
        request,
        timestamp: new Date()
      });

      // Emit error event
      this.emit('payment:error', {
        transactionId,
        error: error.message,
        provider: request.provider || this.defaultProvider
      });

      throw error;
    }
  }

  async refundPayment(
    transactionId: string, 
    amount?: number, 
    reason?: string
  ): Promise<RefundResult> {
    try {
      // Get original transaction
      const transaction = await this.transactionLogger.getTransaction(transactionId);
      if (!transaction) {
        throw new PaymentError('TRANSACTION_NOT_FOUND', `Transaction ${transactionId} not found`);
      }

      // Validate refund amount
      const refundAmount = amount || transaction.amount;
      if (refundAmount > transaction.amount) {
        throw new PaymentError('INVALID_REFUND_AMOUNT', 'Refund amount exceeds original transaction');
      }

      // Get provider
      const provider = this.providers.get(transaction.provider);
      if (!provider) {
        throw new PaymentError('PROVIDER_NOT_FOUND', `Provider ${transaction.provider} not found`);
      }

      // Process refund
      const result = await provider.refundPayment(transaction.providerTransactionId, refundAmount);

      // Log refund
      await this.transactionLogger.logRefund({
        originalTransactionId: transactionId,
        refundId: result.refundId,
        amount: refundAmount,
        reason,
        status: result.status,
        timestamp: new Date()
      });

      // Emit refund event
      this.emit('refund:processed', {
        transactionId,
        refundId: result.refundId,
        amount: refundAmount
      });

      return result;

    } catch (error) {
      this.emit('refund:error', {
        transactionId,
        error: error.message
      });
      throw error;
    }
  }

  async getTransactionHistory(
    customerId: string,
    options: TransactionHistoryOptions = {}
  ): Promise<Transaction[]> {
    const {
      startDate = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000), // 30 days ago
      endDate = new Date(),
      status,
      limit = 100,
      offset = 0
    } = options;

    const transactions = await this.transactionLogger.query({
      customerId,
      startDate,
      endDate,
      status,
      limit,
      offset
    });

    return transactions.map(this.sanitizeTransaction);
  }

  async handleWebhook(
    provider: string,
    payload: any,
    signature: string
  ): Promise<WebhookResult> {
    try {
      // Get provider
      const paymentProvider = this.providers.get(provider);
      if (!paymentProvider) {
        throw new Error(`Unknown provider: ${provider}`);
      }

      // Validate webhook signature
      if (!paymentProvider.validateWebhook(payload, signature)) {
        throw new Error('Invalid webhook signature');
      }

      // Process webhook based on event type
      const result = await this.processWebhookEvent(provider, payload);

      // Log webhook
      await this.transactionLogger.logWebhook({
        provider,
        eventType: payload.type,
        payload,
        result,
        timestamp: new Date()
      });

      return result;

    } catch (error) {
      await this.transactionLogger.logWebhookError({
        provider,
        error: error.message,
        payload,
        timestamp: new Date()
      });
      throw error;
    }
  }

  private async processWebhookEvent(
    provider: string,
    payload: any
  ): Promise<WebhookResult> {
    switch (payload.type) {
      case 'payment.succeeded':
        return this.handlePaymentSuccess(provider, payload);
      
      case 'payment.failed':
        return this.handlePaymentFailure(provider, payload);
      
      case 'refund.completed':
        return this.handleRefundComplete(provider, payload);
      
      case 'dispute.created':
        return this.handleDisputeCreated(provider, payload);
      
      default:
        console.log(`Unhandled webhook event: ${payload.type}`);
        return { processed: false, message: 'Unhandled event type' };
    }
  }

  private validatePaymentRequest(request: PaymentRequest): void {
    if (!request.amount || request.amount <= 0) {
      throw new PaymentError('INVALID_AMOUNT', 'Amount must be greater than 0');
    }

    if (!request.currency || !this.isSupportedCurrency(request.currency)) {
      throw new PaymentError('INVALID_CURRENCY', `Currency ${request.currency} is not supported`);
    }

    if (!request.paymentMethod) {
      throw new PaymentError('MISSING_PAYMENT_METHOD', 'Payment method is required');
    }

    // Validate payment method details
    switch (request.paymentMethod.type) {
      case 'card':
        this.validateCardDetails(request.paymentMethod.card);
        break;
      case 'bank_account':
        this.validateBankAccountDetails(request.paymentMethod.bankAccount);
        break;
      case 'digital_wallet':
        this.validateDigitalWalletDetails(request.paymentMethod.digitalWallet);
        break;
      default:
        throw new PaymentError('INVALID_PAYMENT_METHOD', 'Invalid payment method type');
    }
  }

  private validateCardDetails(card: CardDetails): void {
    if (!card.number || !this.isValidCardNumber(card.number)) {
      throw new PaymentError('INVALID_CARD_NUMBER', 'Invalid card number');
    }

    if (!card.expiryMonth || !card.expiryYear || this.isCardExpired(card)) {
      throw new PaymentError('INVALID_CARD_EXPIRY', 'Invalid or expired card');
    }

    if (!card.cvv || !this.isValidCVV(card.cvv, card.number)) {
      throw new PaymentError('INVALID_CVV', 'Invalid CVV');
    }
  }

  private isValidCardNumber(number: string): boolean {
    // Luhn algorithm validation
    const digits = number.replace(/\D/g, '');
    let sum = 0;
    let isEven = false;

    for (let i = digits.length - 1; i >= 0; i--) {
      let digit = parseInt(digits[i], 10);
      
      if (isEven) {
        digit *= 2;
        if (digit > 9) {
          digit -= 9;
        }
      }
      
      sum += digit;
      isEven = !isEven;
    }

    return sum % 10 === 0;
  }

  private isCardExpired(card: CardDetails): boolean {
    const now = new Date();
    const currentYear = now.getFullYear();
    const currentMonth = now.getMonth() + 1;

    return card.expiryYear < currentYear || 
           (card.expiryYear === currentYear && card.expiryMonth < currentMonth);
  }

  private isValidCVV(cvv: string, cardNumber: string): boolean {
    // American Express has 4-digit CVV, others have 3
    const isAmex = cardNumber.startsWith('34') || cardNumber.startsWith('37');
    const expectedLength = isAmex ? 4 : 3;
    
    return /^\d+$/.test(cvv) && cvv.length === expectedLength;
  }

  private selectProvider(request: PaymentRequest): PaymentProvider {
    const providerName = request.provider || this.defaultProvider;
    const provider = this.providers.get(providerName);

    if (!provider) {
      throw new PaymentError('PROVIDER_NOT_FOUND', `Payment provider ${providerName} not found`);
    }

    return provider;
  }

  private isSupportedCurrency(currency: string): boolean {
    const supportedCurrencies = ['USD', 'EUR', 'GBP', 'CAD', 'AUD', 'JPY'];
    return supportedCurrencies.includes(currency.toUpperCase());
  }

  private sanitizeTransaction(transaction: any): Transaction {
    // Remove sensitive data before returning
    const { cardNumber, cvv, ...sanitized } = transaction;
    
    if (cardNumber) {
      sanitized.cardLast4 = cardNumber.slice(-4);
      sanitized.cardType = this.getCardType(cardNumber);
    }

    return sanitized;
  }

  private getCardType(cardNumber: string): string {
    const patterns = {
      visa: /^4/,
      mastercard: /^5[1-5]/,
      amex: /^3[47]/,
      discover: /^6(?:011|5)/,
    };

    for (const [type, pattern] of Object.entries(patterns)) {
      if (pattern.test(cardNumber)) {
        return type;
      }
    }

    return 'unknown';
  }
}

// Provider implementations
class StripeProvider implements PaymentProvider {
  private stripe: any;
  private config: ProviderConfig;

  constructor(config: ProviderConfig) {
    this.config = config;
    // Initialize Stripe SDK
    // this.stripe = new Stripe(config.apiKey);
  }

  async processPayment(request: PaymentRequest): Promise<PaymentResult> {
    // Stripe-specific implementation
    try {
      const paymentIntent = await this.createPaymentIntent(request);
      const confirmedIntent = await this.confirmPaymentIntent(paymentIntent.id, request);
      
      return {
        status: 'success',
        transactionId: confirmedIntent.id,
        providerTransactionId: confirmedIntent.id,
        amount: request.amount,
        currency: request.currency,
        timestamp: new Date()
      };
    } catch (error) {
      return {
        status: 'failed',
        error: error.message,
        timestamp: new Date()
      };
    }
  }

  async refundPayment(transactionId: string, amount?: number): Promise<RefundResult> {
    // Stripe refund implementation
    return {
      status: 'completed',
      refundId: `refund_${uuidv4()}`,
      amount: amount,
      timestamp: new Date()
    };
  }

  async getTransactionStatus(transactionId: string): Promise<TransactionStatus> {
    // Get status from Stripe
    return {
      status: 'completed',
      transactionId,
      timestamp: new Date()
    };
  }

  validateWebhook(payload: any, signature: string): boolean {
    // Validate Stripe webhook signature
    const expectedSignature = crypto
      .createHmac('sha256', this.config.webhookSecret)
      .update(JSON.stringify(payload))
      .digest('hex');
    
    return signature === expectedSignature;
  }

  private async createPaymentIntent(request: PaymentRequest): Promise<any> {
    // Create Stripe payment intent
    return {
      id: `pi_${uuidv4()}`,
      amount: request.amount,
      currency: request.currency,
      status: 'requires_confirmation'
    };
  }

  private async confirmPaymentIntent(intentId: string, request: PaymentRequest): Promise<any> {
    // Confirm Stripe payment intent
    return {
      id: intentId,
      status: 'succeeded'
    };
  }
}

// Types and interfaces
export interface PaymentRequest {
  amount: number;
  currency: string;
  paymentMethod: PaymentMethod;
  customerId: string;
  orderId?: string;
  description?: string;
  metadata?: Record<string, any>;
  provider?: string;
  savePaymentMethod?: boolean;
}

export interface PaymentMethod {
  type: 'card' | 'bank_account' | 'digital_wallet';
  card?: CardDetails;
  bankAccount?: BankAccountDetails;
  digitalWallet?: DigitalWalletDetails;
}

export interface CardDetails {
  number: string;
  expiryMonth: number;
  expiryYear: number;
  cvv: string;
  holderName: string;
  billingAddress?: Address;
}

export interface PaymentResult {
  status: 'success' | 'failed' | 'pending';
  transactionId?: string;
  providerTransactionId?: string;
  amount?: number;
  currency?: string;
  error?: string;
  timestamp: Date;
}

export interface RefundResult {
  status: 'completed' | 'pending' | 'failed';
  refundId: string;
  amount?: number;
  timestamp: Date;
}

export interface TransactionStatus {
  status: string;
  transactionId: string;
  timestamp: Date;
}

export interface Transaction {
  transactionId: string;
  amount: number;
  currency: string;
  status: string;
  customerId: string;
  timestamp: Date;
  cardLast4?: string;
  cardType?: string;
}

export interface PaymentGatewayConfig {
  defaultProvider: string;
  webhookSecret: string;
  providers: ProviderConfig[];
  rateLimits: RateLimitConfig;
  fraudRules: FraudRule[];
  logPath: string;
}

export interface ProviderConfig {
  name: string;
  apiKey: string;
  webhookSecret: string;
  environment: 'production' | 'sandbox';
}

export class PaymentError extends Error {
  code: string;

  constructor(code: string, message: string) {
    super(message);
    this.code = code;
    this.name = 'PaymentError';
  }
}
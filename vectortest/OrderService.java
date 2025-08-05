// Java Order Service
package com.example.ecommerce.service;

import java.util.*;
import java.math.BigDecimal;
import java.time.LocalDateTime;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Collectors;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

@Service
public class OrderService {
    
    private final OrderRepository orderRepository;
    private final ProductService productService;
    private final PaymentService paymentService;
    private final NotificationService notificationService;
    private final InventoryService inventoryService;
    
    public OrderService(OrderRepository orderRepository, 
                       ProductService productService,
                       PaymentService paymentService,
                       NotificationService notificationService,
                       InventoryService inventoryService) {
        this.orderRepository = orderRepository;
        this.productService = productService;
        this.paymentService = paymentService;
        this.notificationService = notificationService;
        this.inventoryService = inventoryService;
    }
    
    @Transactional
    public Order createOrder(CreateOrderRequest request) throws OrderException {
        // Validate order request
        validateOrderRequest(request);
        
        // Create order entity
        Order order = new Order();
        order.setCustomerId(request.getCustomerId());
        order.setStatus(OrderStatus.PENDING);
        order.setCreatedAt(LocalDateTime.now());
        order.setShippingAddress(request.getShippingAddress());
        
        // Process order items
        List<OrderItem> orderItems = new ArrayList<>();
        BigDecimal totalAmount = BigDecimal.ZERO;
        
        for (OrderItemRequest itemRequest : request.getItems()) {
            Product product = productService.findById(itemRequest.getProductId())
                .orElseThrow(() -> new OrderException("Product not found: " + itemRequest.getProductId()));
            
            // Check inventory
            if (!inventoryService.isAvailable(product.getId(), itemRequest.getQuantity())) {
                throw new OrderException("Insufficient inventory for product: " + product.getName());
            }
            
            OrderItem orderItem = new OrderItem();
            orderItem.setProduct(product);
            orderItem.setQuantity(itemRequest.getQuantity());
            orderItem.setUnitPrice(product.getPrice());
            orderItem.setSubtotal(product.getPrice().multiply(BigDecimal.valueOf(itemRequest.getQuantity())));
            orderItem.setOrder(order);
            
            orderItems.add(orderItem);
            totalAmount = totalAmount.add(orderItem.getSubtotal());
        }
        
        order.setOrderItems(orderItems);
        order.setSubtotal(totalAmount);
        
        // Calculate shipping and tax
        BigDecimal shippingCost = calculateShipping(order);
        BigDecimal tax = calculateTax(order);
        
        order.setShippingCost(shippingCost);
        order.setTax(tax);
        order.setTotalAmount(totalAmount.add(shippingCost).add(tax));
        
        // Reserve inventory
        for (OrderItem item : orderItems) {
            inventoryService.reserve(item.getProduct().getId(), item.getQuantity());
        }
        
        // Save order
        order = orderRepository.save(order);
        
        // Process payment asynchronously
        CompletableFuture<PaymentResult> paymentFuture = CompletableFuture
            .supplyAsync(() -> processPayment(order, request.getPaymentMethod()))
            .exceptionally(ex -> {
                handlePaymentFailure(order, ex);
                return null;
            });
        
        // Send order confirmation
        notificationService.sendOrderConfirmation(order);
        
        return order;
    }
    
    private void validateOrderRequest(CreateOrderRequest request) throws OrderException {
        if (request.getCustomerId() == null) {
            throw new OrderException("Customer ID is required");
        }
        
        if (request.getItems() == null || request.getItems().isEmpty()) {
            throw new OrderException("Order must contain at least one item");
        }
        
        if (request.getShippingAddress() == null) {
            throw new OrderException("Shipping address is required");
        }
        
        // Validate each item
        for (OrderItemRequest item : request.getItems()) {
            if (item.getQuantity() <= 0) {
                throw new OrderException("Invalid quantity for product: " + item.getProductId());
            }
        }
    }
    
    private BigDecimal calculateShipping(Order order) {
        BigDecimal baseShipping = new BigDecimal("10.00");
        BigDecimal itemShipping = new BigDecimal("2.00");
        
        int totalItems = order.getOrderItems().stream()
            .mapToInt(OrderItem::getQuantity)
            .sum();
        
        BigDecimal shippingCost = baseShipping.add(itemShipping.multiply(BigDecimal.valueOf(totalItems)));
        
        // Free shipping for orders over $100
        if (order.getSubtotal().compareTo(new BigDecimal("100.00")) > 0) {
            return BigDecimal.ZERO;
        }
        
        return shippingCost;
    }
    
    private BigDecimal calculateTax(Order order) {
        BigDecimal taxRate = new BigDecimal("0.08"); // 8% tax rate
        return order.getSubtotal().multiply(taxRate).setScale(2, BigDecimal.ROUND_HALF_UP);
    }
    
    private PaymentResult processPayment(Order order, PaymentMethod paymentMethod) {
        try {
            PaymentRequest paymentRequest = new PaymentRequest();
            paymentRequest.setOrderId(order.getId());
            paymentRequest.setAmount(order.getTotalAmount());
            paymentRequest.setPaymentMethod(paymentMethod);
            paymentRequest.setCurrency("USD");
            
            PaymentResult result = paymentService.processPayment(paymentRequest);
            
            if (result.isSuccessful()) {
                order.setStatus(OrderStatus.PAID);
                order.setPaymentId(result.getTransactionId());
                order.setPaidAt(LocalDateTime.now());
            } else {
                order.setStatus(OrderStatus.PAYMENT_FAILED);
            }
            
            orderRepository.save(order);
            return result;
        } catch (Exception e) {
            throw new RuntimeException("Payment processing failed", e);
        }
    }
    
    private void handlePaymentFailure(Order order, Throwable ex) {
        order.setStatus(OrderStatus.PAYMENT_FAILED);
        orderRepository.save(order);
        
        // Release reserved inventory
        for (OrderItem item : order.getOrderItems()) {
            inventoryService.release(item.getProduct().getId(), item.getQuantity());
        }
        
        // Notify customer
        notificationService.sendPaymentFailureNotification(order, ex.getMessage());
    }
    
    public Order findById(Long orderId) {
        return orderRepository.findById(orderId)
            .orElseThrow(() -> new OrderNotFoundException("Order not found: " + orderId));
    }
    
    public List<Order> findByCustomerId(Long customerId) {
        return orderRepository.findByCustomerId(customerId);
    }
    
    public List<Order> findByStatus(OrderStatus status) {
        return orderRepository.findByStatus(status);
    }
    
    @Transactional
    public Order updateOrderStatus(Long orderId, OrderStatus newStatus) {
        Order order = findById(orderId);
        OrderStatus oldStatus = order.getStatus();
        
        // Validate status transition
        if (!isValidStatusTransition(oldStatus, newStatus)) {
            throw new OrderException("Invalid status transition from " + oldStatus + " to " + newStatus);
        }
        
        order.setStatus(newStatus);
        order.setUpdatedAt(LocalDateTime.now());
        
        // Handle status-specific logic
        switch (newStatus) {
            case SHIPPED:
                order.setShippedAt(LocalDateTime.now());
                notificationService.sendShippingNotification(order);
                break;
            case DELIVERED:
                order.setDeliveredAt(LocalDateTime.now());
                notificationService.sendDeliveryNotification(order);
                break;
            case CANCELLED:
                handleOrderCancellation(order);
                break;
        }
        
        return orderRepository.save(order);
    }
    
    private boolean isValidStatusTransition(OrderStatus from, OrderStatus to) {
        Map<OrderStatus, Set<OrderStatus>> validTransitions = new HashMap<>();
        validTransitions.put(OrderStatus.PENDING, Set.of(OrderStatus.PAID, OrderStatus.CANCELLED));
        validTransitions.put(OrderStatus.PAID, Set.of(OrderStatus.PROCESSING, OrderStatus.CANCELLED));
        validTransitions.put(OrderStatus.PROCESSING, Set.of(OrderStatus.SHIPPED, OrderStatus.CANCELLED));
        validTransitions.put(OrderStatus.SHIPPED, Set.of(OrderStatus.DELIVERED, OrderStatus.RETURNED));
        validTransitions.put(OrderStatus.DELIVERED, Set.of(OrderStatus.RETURNED));
        
        return validTransitions.getOrDefault(from, Set.of()).contains(to);
    }
    
    private void handleOrderCancellation(Order order) {
        // Release inventory
        for (OrderItem item : order.getOrderItems()) {
            inventoryService.release(item.getProduct().getId(), item.getQuantity());
        }
        
        // Process refund if payment was made
        if (order.getPaymentId() != null) {
            RefundRequest refundRequest = new RefundRequest();
            refundRequest.setPaymentId(order.getPaymentId());
            refundRequest.setAmount(order.getTotalAmount());
            refundRequest.setReason("Order cancelled");
            
            paymentService.processRefund(refundRequest);
        }
        
        // Send cancellation notification
        notificationService.sendCancellationNotification(order);
    }
    
    public OrderStatistics getOrderStatistics(LocalDateTime startDate, LocalDateTime endDate) {
        List<Order> orders = orderRepository.findByCreatedAtBetween(startDate, endDate);
        
        OrderStatistics stats = new OrderStatistics();
        stats.setTotalOrders(orders.size());
        stats.setTotalRevenue(orders.stream()
            .filter(o -> o.getStatus() != OrderStatus.CANCELLED)
            .map(Order::getTotalAmount)
            .reduce(BigDecimal.ZERO, BigDecimal::add));
        
        Map<OrderStatus, Long> statusCounts = orders.stream()
            .collect(Collectors.groupingBy(Order::getStatus, Collectors.counting()));
        stats.setOrdersByStatus(statusCounts);
        
        return stats;
    }
}
# Contributing to E-commerce Platform

First off, thank you for considering contributing to our e-commerce platform! It's people like you that make this project such a great tool. We welcome contributions from everyone, regardless of their level of experience.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Process](#development-process)
- [Style Guidelines](#style-guidelines)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)
- [Community](#community)

## Code of Conduct

This project and everyone participating in it is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to conduct@example.com.

### Our Standards

- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

## Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:

```bash
# Required versions
Node.js: 18.x or higher
npm: 9.x or higher
Git: 2.30 or higher
Docker: 20.10 or higher
Docker Compose: 2.0 or higher
```

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:

```bash
git clone https://github.com/your-username/ecommerce-platform.git
cd ecommerce-platform
```

3. Add the upstream remote:

```bash
git remote add upstream https://github.com/original-owner/ecommerce-platform.git
```

### Development Environment Setup

1. **Install dependencies:**

```bash
# Install all dependencies
npm install

# Install pre-commit hooks
npm run prepare
```

2. **Set up environment variables:**

```bash
# Copy the example environment file
cp .env.example .env.local

# Edit .env.local with your local configuration
```

3. **Start the development environment:**

```bash
# Start all services using Docker Compose
docker-compose up -d

# Run database migrations
npm run migrate:dev

# Seed the database with test data
npm run seed:dev
```

4. **Verify the setup:**

```bash
# Run the test suite
npm test

# Start the development server
npm run dev
```

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the existing issues to avoid duplicates. When you create a bug report, include as many details as possible:

**Bug Report Template:**

```markdown
### Description
[Clear and concise description of the bug]

### Steps to Reproduce
1. Go to '...'
2. Click on '...'
3. Scroll down to '...'
4. See error

### Expected Behavior
[What you expected to happen]

### Actual Behavior
[What actually happened]

### Screenshots
[If applicable, add screenshots]

### Environment
- OS: [e.g., macOS 12.6]
- Browser: [e.g., Chrome 109]
- Node.js version: [e.g., 18.12.0]
- npm version: [e.g., 9.2.0]

### Additional Context
[Any other relevant information]
```

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, include:

**Enhancement Request Template:**

```markdown
### Problem Statement
[Describe the problem or limitation you're trying to solve]

### Proposed Solution
[Clear description of the suggested enhancement]

### Alternatives Considered
[Any alternative solutions or features you've considered]

### Additional Context
[Mockups, examples, or reference implementations]
```

### Code Contributions

#### Finding Issues to Work On

- Look for issues labeled `good first issue` for beginner-friendly tasks
- Check issues labeled `help wanted` for more complex contributions
- Review the `bug` label for bug fixes
- Explore `enhancement` labeled issues for new features

#### Before Starting Work

1. **Check if someone is already working on it:**
   - Look for assignees on the issue
   - Check comments for "I'm working on this"
   - Ask to be assigned if no one is working on it

2. **Discuss major changes:**
   - For significant changes, open an issue first
   - Get feedback on your approach
   - Ensure alignment with project goals

## Development Process

### Branch Naming Convention

Use descriptive branch names following this pattern:

```
type/issue-number-brief-description

Examples:
- feature/123-add-payment-gateway
- fix/456-cart-calculation-error
- docs/789-update-api-documentation
- refactor/321-optimize-database-queries
```

### Development Workflow

1. **Create a feature branch:**

```bash
git checkout -b feature/123-add-payment-gateway
```

2. **Make your changes:**
   - Write clean, maintainable code
   - Follow the style guidelines
   - Add/update tests as needed
   - Update documentation

3. **Test your changes:**

```bash
# Run unit tests
npm test

# Run integration tests
npm run test:integration

# Run linting
npm run lint

# Run type checking
npm run type-check
```

4. **Commit your changes:**
   - Follow the commit message guidelines
   - Make small, focused commits
   - Write clear commit messages

5. **Keep your branch updated:**

```bash
git fetch upstream
git rebase upstream/main
```

## Style Guidelines

### JavaScript/TypeScript

We use ESLint and Prettier for code formatting. Configuration is in `.eslintrc.js` and `.prettierrc`.

**Key Guidelines:**

```javascript
// Use meaningful variable names
const userAuthenticationToken = generateToken(); // Good
const token = genTok(); // Bad

// Use async/await over promises
// Good
async function fetchUser(id) {
  try {
    const user = await db.users.findById(id);
    return user;
  } catch (error) {
    logger.error('Failed to fetch user:', error);
    throw error;
  }
}

// Bad
function fetchUser(id) {
  return db.users.findById(id)
    .then(user => user)
    .catch(error => {
      logger.error('Failed to fetch user:', error);
      throw error;
    });
}

// Use functional programming where appropriate
const activeUsers = users
  .filter(user => user.isActive)
  .map(user => ({
    id: user.id,
    name: user.name,
    email: user.email
  }));

// Document complex logic
/**
 * Calculates the total price including tax and shipping
 * @param {number} subtotal - The subtotal before tax and shipping
 * @param {number} taxRate - The tax rate as a decimal (e.g., 0.08 for 8%)
 * @param {number} shippingCost - The shipping cost
 * @returns {number} The total price rounded to 2 decimal places
 */
function calculateTotal(subtotal, taxRate, shippingCost) {
  const tax = subtotal * taxRate;
  const total = subtotal + tax + shippingCost;
  return Math.round(total * 100) / 100;
}
```

### Python

Follow PEP 8 with Black formatter. Configuration is in `pyproject.toml`.

```python
# Use type hints
from typing import List, Optional, Dict

def process_order(
    order_id: str,
    items: List[Dict[str, Any]],
    user_id: Optional[str] = None
) -> OrderResult:
    """
    Process an order and return the result.
    
    Args:
        order_id: Unique identifier for the order
        items: List of order items
        user_id: Optional user identifier
        
    Returns:
        OrderResult object containing processing details
        
    Raises:
        ValidationError: If order data is invalid
        ProcessingError: If order processing fails
    """
    # Implementation here
    pass

# Use descriptive names and docstrings
class PaymentProcessor:
    """Handles payment processing for various payment methods."""
    
    def __init__(self, config: PaymentConfig):
        """
        Initialize the payment processor.
        
        Args:
            config: Payment configuration object
        """
        self.config = config
        self._providers = self._initialize_providers()
```

### Go

Follow the official Go style guide and use `gofmt`.

```go
// Package payment handles payment processing
package payment

import (
    "context"
    "fmt"
    "time"
)

// Processor handles payment transactions
type Processor struct {
    providers map[string]Provider
    logger    Logger
}

// ProcessPayment processes a payment transaction
func (p *Processor) ProcessPayment(ctx context.Context, req *PaymentRequest) (*PaymentResult, error) {
    // Validate request
    if err := req.Validate(); err != nil {
        return nil, fmt.Errorf("invalid payment request: %w", err)
    }
    
    // Get provider
    provider, ok := p.providers[req.Provider]
    if !ok {
        return nil, ErrProviderNotFound
    }
    
    // Process payment with timeout
    ctx, cancel := context.WithTimeout(ctx, 30*time.Second)
    defer cancel()
    
    result, err := provider.Process(ctx, req)
    if err != nil {
        p.logger.Error("payment processing failed", 
            "provider", req.Provider,
            "error", err,
        )
        return nil, err
    }
    
    return result, nil
}
```

### SQL

Use consistent formatting and meaningful names:

```sql
-- Use uppercase for SQL keywords
-- Use snake_case for identifiers
-- Add comments for complex queries

-- Create index for frequently queried columns
CREATE INDEX idx_orders_user_created 
ON orders(user_id, created_at DESC)
WHERE status != 'cancelled';

-- Use CTEs for complex queries
WITH monthly_revenue AS (
    SELECT 
        DATE_TRUNC('month', created_at) AS month,
        SUM(total_amount) AS revenue,
        COUNT(DISTINCT user_id) AS unique_customers
    FROM orders
    WHERE status = 'completed'
        AND created_at >= CURRENT_DATE - INTERVAL '12 months'
    GROUP BY DATE_TRUNC('month', created_at)
),
revenue_growth AS (
    SELECT 
        month,
        revenue,
        unique_customers,
        LAG(revenue) OVER (ORDER BY month) AS prev_month_revenue,
        (revenue - LAG(revenue) OVER (ORDER BY month)) / 
            NULLIF(LAG(revenue) OVER (ORDER BY month), 0) * 100 AS growth_rate
    FROM monthly_revenue
)
SELECT * FROM revenue_growth
ORDER BY month DESC;
```

## Commit Guidelines

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification.

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that don't affect code meaning (formatting, etc.)
- **refactor**: Code changes that neither fix bugs nor add features
- **perf**: Performance improvements
- **test**: Adding or updating tests
- **build**: Changes to the build system or dependencies
- **ci**: Changes to CI configuration files and scripts
- **chore**: Other changes that don't modify src or test files

### Examples

```bash
# Feature
feat(auth): add two-factor authentication

Implement TOTP-based 2FA with QR code generation.
Users can now enable 2FA from their security settings.

Closes #123

# Bug fix
fix(cart): correct tax calculation for international orders

Tax was being calculated before applying international
shipping rates, resulting in incorrect totals.

# Documentation
docs(api): update authentication endpoint documentation

- Add examples for OAuth2 flow
- Update error response codes
- Add rate limiting information

# Performance
perf(search): optimize product search query

- Add compound index on (category_id, price, rating)
- Implement query result caching
- Reduce N+1 queries in search results

Improves search response time by ~40%
```

## Pull Request Process

### Before Submitting

1. **Ensure all tests pass:**

```bash
npm test
npm run test:integration
npm run lint
```

2. **Update documentation:**
   - API documentation for endpoint changes
   - README for significant features
   - Code comments for complex logic

3. **Self-review your code:**
   - Check for typos and formatting
   - Ensure no debug code is left
   - Verify all edge cases are handled

### Pull Request Template

```markdown
## Description
[Provide a brief description of the changes]

## Type of Change
- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Related Issues
Closes #[issue number]

## Testing
- [ ] Unit tests pass locally
- [ ] Integration tests pass locally
- [ ] Manual testing completed

## Screenshots
[If applicable, add screenshots of UI changes]

## Checklist
- [ ] My code follows the project's style guidelines
- [ ] I have performed a self-review
- [ ] I have added tests that prove my fix/feature works
- [ ] New and existing unit tests pass locally
- [ ] I have updated the documentation accordingly
- [ ] My changes generate no new warnings
```

### Review Process

1. **Automated checks:**
   - CI/CD pipeline runs tests
   - Code coverage is maintained
   - Linting passes

2. **Code review:**
   - At least one maintainer approval required
   - Address all feedback constructively
   - Update PR based on suggestions

3. **Merge requirements:**
   - All checks must pass
   - Branch must be up to date with main
   - No merge conflicts

## Testing Guidelines

### Unit Tests

```javascript
// Good unit test example
describe('OrderService', () => {
  let orderService;
  let mockDatabase;
  let mockPaymentGateway;

  beforeEach(() => {
    mockDatabase = createMockDatabase();
    mockPaymentGateway = createMockPaymentGateway();
    orderService = new OrderService(mockDatabase, mockPaymentGateway);
  });

  describe('createOrder', () => {
    it('should create order with valid data', async () => {
      // Arrange
      const orderData = {
        userId: 'user-123',
        items: [
          { productId: 'prod-456', quantity: 2, price: 29.99 }
        ]
      };
      mockDatabase.users.findById.mockResolvedValue({ id: 'user-123' });
      mockDatabase.products.findById.mockResolvedValue({ id: 'prod-456', stock: 10 });
      mockPaymentGateway.charge.mockResolvedValue({ success: true, transactionId: 'txn-789' });

      // Act
      const order = await orderService.createOrder(orderData);

      // Assert
      expect(order).toMatchObject({
        userId: 'user-123',
        status: 'completed',
        total: 59.98
      });
      expect(mockDatabase.orders.create).toHaveBeenCalledWith(
        expect.objectContaining({
          userId: 'user-123',
          items: expect.arrayContaining([
            expect.objectContaining({ productId: 'prod-456' })
          ])
        })
      );
    });

    it('should throw error when user not found', async () => {
      // Arrange
      const orderData = { userId: 'invalid-user' };
      mockDatabase.users.findById.mockResolvedValue(null);

      // Act & Assert
      await expect(orderService.createOrder(orderData))
        .rejects
        .toThrow('User not found');
    });
  });
});
```

### Integration Tests

```javascript
// Integration test example
describe('Order API Integration', () => {
  let app;
  let authToken;

  beforeAll(async () => {
    app = await createTestApp();
    await seedTestDatabase();
    authToken = await getTestAuthToken();
  });

  afterAll(async () => {
    await cleanupTestDatabase();
    await app.close();
  });

  describe('POST /api/orders', () => {
    it('should create order successfully', async () => {
      const response = await request(app)
        .post('/api/orders')
        .set('Authorization', `Bearer ${authToken}`)
        .send({
          items: [
            { productId: 'test-product-1', quantity: 2 }
          ],
          shippingAddress: {
            street: '123 Test St',
            city: 'Test City',
            zipCode: '12345'
          }
        });

      expect(response.status).toBe(201);
      expect(response.body).toHaveProperty('orderId');
      expect(response.body.status).toBe('pending');
    });
  });
});
```

## Documentation

### Code Documentation

- Add JSDoc comments for all public APIs
- Include examples in complex functions
- Document edge cases and assumptions

```javascript
/**
 * Calculates shipping cost based on order weight and destination
 * 
 * @param {Object} options - Shipping calculation options
 * @param {number} options.weight - Total weight in kg
 * @param {string} options.destinationCountry - ISO country code
 * @param {string} options.shippingMethod - One of: 'standard', 'express', 'overnight'
 * @param {boolean} [options.isFragile=false] - Whether the package contains fragile items
 * 
 * @returns {Object} Shipping cost and estimated delivery
 * @returns {number} returns.cost - Shipping cost in USD
 * @returns {Date} returns.estimatedDelivery - Estimated delivery date
 * 
 * @throws {ValidationError} If weight is negative or country code is invalid
 * 
 * @example
 * const shipping = calculateShipping({
 *   weight: 2.5,
 *   destinationCountry: 'US',
 *   shippingMethod: 'express'
 * });
 * // Returns: { cost: 15.99, estimatedDelivery: Date }
 */
function calculateShipping(options) {
  // Implementation
}
```

### API Documentation

Update OpenAPI/Swagger documentation for any API changes:

```yaml
paths:
  /api/products/{id}:
    get:
      summary: Get product by ID
      description: Retrieves detailed information about a specific product
      operationId: getProductById
      tags:
        - Products
      parameters:
        - name: id
          in: path
          required: true
          description: Product unique identifier
          schema:
            type: string
            format: uuid
      responses:
        '200':
          description: Product found
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Product'
              example:
                id: "123e4567-e89b-12d3-a456-426614174000"
                name: "Wireless Headphones"
                price: 149.99
                inStock: true
        '404':
          description: Product not found
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
```

## Community

### Communication Channels

- **Discord**: [Join our Discord server](https://discord.gg/example)
- **Forum**: [Community Forum](https://forum.example.com)
- **Twitter**: [@example_platform](https://twitter.com/example_platform)
- **Blog**: [Engineering Blog](https://blog.example.com)

### Getting Help

- Check the [FAQ](FAQ.md) first
- Search existing issues on GitHub
- Ask in the #help channel on Discord
- Create a detailed issue if you can't find a solution

### Recognition

We believe in recognizing contributors:

- Contributors are listed in [CONTRIBUTORS.md](CONTRIBUTORS.md)
- Significant contributors get a special badge
- Monthly contributor spotlight in our newsletter
- Annual contributor summit (expenses covered)

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT License).

## Questions?

If you have questions about the contribution process, feel free to:
- Ask in our Discord #contributors channel
- Email the maintainers at maintainers@example.com
- Open a discussion on GitHub

Thank you for contributing to our project! 🎉
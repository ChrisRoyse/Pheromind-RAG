# API Documentation

## Overview

This document provides comprehensive documentation for the E-commerce Platform REST API. The API follows RESTful principles and uses JSON for request and response payloads.

## Base URL

```
https://api.example.com/v1
```

## Authentication

The API uses JWT (JSON Web Tokens) for authentication. Include the token in the Authorization header:

```
Authorization: Bearer <your-jwt-token>
```

### Obtaining a Token

```http
POST /auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "your-password"
}
```

Response:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "email": "user@example.com",
    "role": "customer"
  },
  "expiresIn": 86400
}
```

## Error Handling

The API uses standard HTTP status codes and returns error details in the response body:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input data",
    "details": [
      {
        "field": "email",
        "message": "Invalid email format"
      }
    ]
  },
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### Common Error Codes

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 400 | BAD_REQUEST | Invalid request format or parameters |
| 401 | UNAUTHORIZED | Missing or invalid authentication token |
| 403 | FORBIDDEN | Insufficient permissions |
| 404 | NOT_FOUND | Resource not found |
| 409 | CONFLICT | Resource already exists |
| 422 | VALIDATION_ERROR | Input validation failed |
| 429 | RATE_LIMIT_EXCEEDED | Too many requests |
| 500 | INTERNAL_ERROR | Server error |

## Rate Limiting

API requests are limited to:
- 100 requests per minute for authenticated users
- 20 requests per minute for unauthenticated users

Rate limit information is included in response headers:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1642248000
```

## Endpoints

### Users

#### Create User Account

```http
POST /users
Content-Type: application/json

{
  "username": "johndoe",
  "email": "john@example.com",
  "password": "SecurePassword123!",
  "firstName": "John",
  "lastName": "Doe"
}
```

#### Get User Profile

```http
GET /users/{userId}
Authorization: Bearer <token>
```

Response:
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "username": "johndoe",
  "email": "john@example.com",
  "firstName": "John",
  "lastName": "Doe",
  "createdAt": "2024-01-15T10:30:00Z",
  "role": "customer",
  "isActive": true,
  "isVerified": true
}
```

#### Update User Profile

```http
PUT /users/{userId}
Authorization: Bearer <token>
Content-Type: application/json

{
  "firstName": "John",
  "lastName": "Smith",
  "phoneNumber": "+1234567890"
}
```

### Products

#### List Products

```http
GET /products?page=1&limit=20&category=electronics&sort=price_asc
```

Query Parameters:
- `page` (integer): Page number (default: 1)
- `limit` (integer): Items per page (default: 20, max: 100)
- `category` (string): Filter by category slug
- `brand` (string): Filter by brand
- `minPrice` (number): Minimum price filter
- `maxPrice` (number): Maximum price filter
- `inStock` (boolean): Filter in-stock items only
- `search` (string): Search query
- `sort` (string): Sort order (price_asc, price_desc, name_asc, name_desc, rating, newest)

Response:
```json
{
  "data": [
    {
      "id": "prod_123",
      "sku": "ELEC-001",
      "name": "Wireless Headphones",
      "slug": "wireless-headphones",
      "description": "High-quality wireless headphones with noise cancellation",
      "category": {
        "id": 5,
        "name": "Electronics",
        "slug": "electronics"
      },
      "brand": "TechBrand",
      "price": 149.99,
      "salePrice": 119.99,
      "currency": "USD",
      "inStock": true,
      "stockQuantity": 50,
      "images": [
        {
          "url": "https://cdn.example.com/products/headphones-1.jpg",
          "alt": "Product front view",
          "isPrimary": true
        }
      ],
      "rating": 4.5,
      "reviewCount": 234
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 150,
    "totalPages": 8
  }
}
```

#### Get Product Details

```http
GET /products/{productId}
```

#### Create Product (Admin Only)

```http
POST /products
Authorization: Bearer <admin-token>
Content-Type: application/json

{
  "sku": "ELEC-002",
  "name": "Smart Watch",
  "description": "Feature-rich smartwatch with health tracking",
  "categoryId": 5,
  "brand": "TechBrand",
  "price": 299.99,
  "stockQuantity": 100,
  "weight": 0.05,
  "dimensions": {
    "length": 4.5,
    "width": 3.8,
    "height": 1.2,
    "unit": "cm"
  }
}
```

### Orders

#### Create Order

```http
POST /orders
Authorization: Bearer <token>
Content-Type: application/json

{
  "items": [
    {
      "productId": "prod_123",
      "quantity": 2
    }
  ],
  "shippingAddress": {
    "firstName": "John",
    "lastName": "Doe",
    "street": "123 Main St",
    "city": "New York",
    "state": "NY",
    "zipCode": "10001",
    "country": "US"
  },
  "billingAddress": {
    "sameAsShipping": true
  },
  "paymentMethod": {
    "type": "card",
    "card": {
      "number": "4111111111111111",
      "expiryMonth": 12,
      "expiryYear": 2025,
      "cvv": "123",
      "holderName": "John Doe"
    }
  }
}
```

#### Get Order Status

```http
GET /orders/{orderId}
Authorization: Bearer <token>
```

#### List User Orders

```http
GET /orders?status=delivered&startDate=2024-01-01&endDate=2024-01-31
Authorization: Bearer <token>
```

### Shopping Cart

#### Get Cart

```http
GET /cart
Authorization: Bearer <token>
```

#### Add to Cart

```http
POST /cart/items
Authorization: Bearer <token>
Content-Type: application/json

{
  "productId": "prod_123",
  "quantity": 1
}
```

#### Update Cart Item

```http
PUT /cart/items/{itemId}
Authorization: Bearer <token>
Content-Type: application/json

{
  "quantity": 3
}
```

#### Remove from Cart

```http
DELETE /cart/items/{itemId}
Authorization: Bearer <token>
```

#### Clear Cart

```http
DELETE /cart
Authorization: Bearer <token>
```

### Reviews

#### Get Product Reviews

```http
GET /products/{productId}/reviews?page=1&limit=10&sort=helpful
```

#### Create Review

```http
POST /products/{productId}/reviews
Authorization: Bearer <token>
Content-Type: application/json

{
  "rating": 5,
  "title": "Excellent product!",
  "comment": "Very satisfied with this purchase. Great quality and fast shipping."
}
```

## Webhooks

Configure webhooks to receive real-time notifications about events:

### Available Events

- `order.created`
- `order.paid`
- `order.shipped`
- `order.delivered`
- `order.cancelled`
- `payment.succeeded`
- `payment.failed`
- `product.created`
- `product.updated`
- `product.deleted`
- `review.created`

### Webhook Payload

```json
{
  "id": "evt_123",
  "type": "order.created",
  "data": {
    "orderId": "ord_456",
    "amount": 299.99,
    "currency": "USD",
    "customerId": "cust_789"
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "signature": "sha256=..."
}
```

### Webhook Security

Verify webhook signatures using HMAC-SHA256:

```javascript
const crypto = require('crypto');

function verifyWebhookSignature(payload, signature, secret) {
  const expectedSignature = crypto
    .createHmac('sha256', secret)
    .update(JSON.stringify(payload))
    .digest('hex');
  
  return `sha256=${expectedSignature}` === signature;
}
```

## Pagination

All list endpoints support pagination using the following parameters:

- `page`: Page number (starts at 1)
- `limit`: Number of items per page (max: 100)

Pagination metadata is included in responses:

```json
{
  "data": [...],
  "pagination": {
    "page": 2,
    "limit": 20,
    "total": 150,
    "totalPages": 8,
    "hasNext": true,
    "hasPrev": true
  }
}
```

## Search

### Product Search

```http
GET /search/products?q=wireless+headphones&filters[category]=electronics&filters[priceRange]=50-200
```

### Autocomplete

```http
GET /search/autocomplete?q=wire
```

Response:
```json
{
  "suggestions": [
    {
      "text": "wireless headphones",
      "category": "Electronics",
      "type": "product"
    },
    {
      "text": "wireless mouse",
      "category": "Electronics",
      "type": "product"
    }
  ]
}
```

## SDK Examples

### JavaScript/TypeScript

```typescript
import { EcommerceAPI } from '@example/ecommerce-sdk';

const api = new EcommerceAPI({
  baseURL: 'https://api.example.com/v1',
  apiKey: 'your-api-key'
});

// Get products
const products = await api.products.list({
  category: 'electronics',
  limit: 20
});

// Create order
const order = await api.orders.create({
  items: [
    { productId: 'prod_123', quantity: 1 }
  ],
  shippingAddress: {...}
});
```

### Python

```python
from ecommerce_sdk import EcommerceAPI

api = EcommerceAPI(
    base_url='https://api.example.com/v1',
    api_key='your-api-key'
)

# Get products
products = api.products.list(
    category='electronics',
    limit=20
)

# Create order
order = api.orders.create(
    items=[
        {'product_id': 'prod_123', 'quantity': 1}
    ],
    shipping_address={...}
)
```

## Testing

Use our sandbox environment for testing:

- Base URL: `https://sandbox-api.example.com/v1`
- Test API Key: `test_key_123456789`
- Test Credit Card: `4111 1111 1111 1111`

## Support

- Email: api-support@example.com
- Documentation: https://docs.example.com
- Status Page: https://status.example.com
- Developer Forum: https://forum.example.com
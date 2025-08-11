# Sample Markdown Document

This is a comprehensive test document that contains various markdown elements.

## Introduction

Markdown is a lightweight markup language that allows you to format text using plain text syntax.

### Key Features

- **Simple syntax**: Easy to read and write
- **Wide support**: Works across many platforms
- **Flexible**: Can be converted to HTML, PDF, etc.

## Code Examples

### JavaScript Function

Here's a simple JavaScript function:

```javascript
function calculateSum(a, b) {
    if (typeof a !== 'number' || typeof b !== 'number') {
        throw new Error('Both arguments must be numbers');
    }
    return a + b;
}

// Usage example
const result = calculateSum(5, 3);
console.log('Sum:', result);
```

### Python Class

And here's a Python class:

```python
class Calculator:
    def __init__(self):
        self.history = []
    
    def add(self, x, y):
        result = x + y
        self.history.append(f"{x} + {y} = {result}")
        return result
    
    def get_history(self):
        return self.history.copy()
```

## Data Tables

### Performance Comparison

| Operation | Time (ms) | Memory (MB) | CPU (%) |
|-----------|-----------|-------------|---------|
| Read      | 10        | 5           | 15      |
| Write     | 25        | 8           | 35      |
| Delete    | 15        | 3           | 20      |
| Update    | 20        | 6           | 30      |

### Feature Matrix

| Feature     | Basic | Pro | Enterprise |
|-------------|-------|-----|------------|
| Users       | 5     | 25  | Unlimited  |
| Storage     | 1GB   | 10GB| 100GB      |
| Support     | Email | Chat| Phone      |
| API Access  | No    | Yes | Yes        |
| Analytics   | No    | Basic| Advanced  |

## Lists and Tasks

### Project Checklist

1. **Planning Phase**
   - [ ] Define requirements
   - [ ] Create wireframes
   - [ ] Set up development environment
   - [x] Choose technology stack

2. **Development Phase** 
   - [x] Set up repository
   - [x] Create initial structure
   - [ ] Implement core features
     - [x] User authentication
     - [ ] Data processing
     - [ ] API endpoints
   - [ ] Write tests
   
3. **Deployment Phase**
   - [ ] Set up CI/CD pipeline
   - [ ] Configure production environment
   - [ ] Deploy application
   - [ ] Monitor performance

### Shopping List

- Groceries
  - Milk (2%)
  - Bread (whole wheat)
  - Eggs (dozen)
  - Bananas
  - Apples (Gala)
- Household
  - Toilet paper
  - Dish soap
  - Laundry detergent
- Electronics
  - USB cables
  - Phone charger
  - Batteries (AA)

## Complex Nested Content

### API Documentation

The following example shows how to use our REST API:

#### Authentication

First, obtain an access token:

```bash
curl -X POST https://api.example.com/auth/token \
  -H "Content-Type: application/json" \
  -d '{
    "username": "your_username",
    "password": "your_password"
  }'
```

Response:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

#### Making API Calls

Use the token in subsequent requests:

```bash
curl -X GET https://api.example.com/users/me \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN"
```

#### Error Handling

Our API returns standard HTTP status codes:

| Code | Status | Description |
|------|--------|-------------|
| 200  | OK     | Success     |
| 400  | Bad Request | Invalid input |
| 401  | Unauthorized | Missing/invalid token |
| 403  | Forbidden | Insufficient permissions |
| 404  | Not Found | Resource doesn't exist |
| 500  | Internal Error | Server problem |

Example error response:
```json
{
  "error": {
    "code": "INVALID_INPUT",
    "message": "The provided email address is not valid",
    "details": {
      "field": "email",
      "value": "invalid-email"
    }
  }
}
```

## Mathematical Formulas

The quadratic formula is:

**x = (-b ± √(b² - 4ac)) / 2a**

For calculating compound interest:

**A = P(1 + r/n)^(nt)**

Where:
- A = final amount
- P = principal amount
- r = annual interest rate (decimal)
- n = number of times interest is compounded per year
- t = number of years

## Quotes and References

> "The best way to predict the future is to invent it."
> 
> — Alan Kay

> "Programs must be written for people to read, and only incidentally 
> for machines to execute."
> 
> — Harold Abelson

### References

1. Smith, J. (2023). *Modern Software Development Practices*. Tech Publishing.
2. Johnson, M. & Davis, L. (2022). "API Design Patterns." *Journal of Software Engineering*, 45(3), 123-145.
3. Brown, A. (2024). [Understanding Markdown Syntax](https://example.com/markdown-guide). Retrieved March 15, 2024.

## Conclusion

This document demonstrates various markdown elements that should be properly handled by the chunking system. Each section tests different aspects:

- **Headers**: Multiple levels of hierarchy
- **Code blocks**: Different languages and inline code
- **Tables**: Data presentation with alignment
- **Lists**: Ordered, unordered, nested, and task lists
- **Mixed content**: Complex combinations of elements
- **Special characters**: Formulas, quotes, and symbols

The chunking system should preserve the semantic structure while creating manageable, searchable segments of content.

---

*Document generated for testing purposes - Version 1.0*
package models

import (
    "time"
    "encoding/json"
    "database/sql"
)

// User represents a system user
type User struct {
    ID        int64     `json:"id" db:"id"`
    Username  string    `json:"username" db:"username"`
    Email     string    `json:"email" db:"email"`
    Password  string    `json:"-" db:"password_hash"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    UpdatedAt time.Time `json:"updated_at" db:"updated_at"`
    IsActive  bool      `json:"is_active" db:"is_active"`
    Role      string    `json:"role" db:"role"`
}

// Product represents an item in our catalog
type Product struct {
    ID          int64           `json:"id"`
    Name        string          `json:"name"`
    Description string          `json:"description"`
    Price       float64         `json:"price"`
    Stock       int             `json:"stock"`
    CategoryID  int64           `json:"category_id"`
    Images      []string        `json:"images"`
    Attributes  json.RawMessage `json:"attributes"`
    CreatedAt   time.Time       `json:"created_at"`
    UpdatedAt   time.Time       `json:"updated_at"`
}

// Order represents a customer order
type Order struct {
    ID         int64       `json:"id"`
    UserID     int64       `json:"user_id"`
    Items      []OrderItem `json:"items"`
    Total      float64     `json:"total"`
    Status     string      `json:"status"`
    PaymentID  string      `json:"payment_id"`
    ShippingID string      `json:"shipping_id"`
    CreatedAt  time.Time   `json:"created_at"`
    UpdatedAt  time.Time   `json:"updated_at"`
}

type OrderItem struct {
    ProductID int64   `json:"product_id"`
    Quantity  int     `json:"quantity"`
    Price     float64 `json:"price"`
}

// Repository handles database operations
type Repository struct {
    db *sql.DB
}

func NewRepository(db *sql.DB) *Repository {
    return &Repository{db: db}
}

func (r *Repository) GetUserByID(id int64) (*User, error) {
    var user User
    query := `SELECT id, username, email, password_hash, created_at, updated_at, is_active, role 
              FROM users WHERE id = $1`
    
    err := r.db.QueryRow(query, id).Scan(
        &user.ID, &user.Username, &user.Email, &user.Password,
        &user.CreatedAt, &user.UpdatedAt, &user.IsActive, &user.Role,
    )
    
    if err != nil {
        return nil, err
    }
    
    return &user, nil
}

func (r *Repository) CreateOrder(order *Order) error {
    tx, err := r.db.Begin()
    if err != nil {
        return err
    }
    defer tx.Rollback()
    
    query := `INSERT INTO orders (user_id, total, status, payment_id, shipping_id, created_at, updated_at)
              VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id`
    
    err = tx.QueryRow(query, order.UserID, order.Total, order.Status,
        order.PaymentID, order.ShippingID, time.Now(), time.Now()).Scan(&order.ID)
    
    if err != nil {
        return err
    }
    
    // Insert order items
    for _, item := range order.Items {
        itemQuery := `INSERT INTO order_items (order_id, product_id, quantity, price)
                      VALUES ($1, $2, $3, $4)`
        _, err = tx.Exec(itemQuery, order.ID, item.ProductID, item.Quantity, item.Price)
        if err != nil {
            return err
        }
    }
    
    return tx.Commit()
}

func (r *Repository) SearchProducts(query string, limit int) ([]Product, error) {
    searchQuery := `SELECT id, name, description, price, stock, category_id, created_at, updated_at
                    FROM products 
                    WHERE name ILIKE $1 OR description ILIKE $1
                    LIMIT $2`
    
    rows, err := r.db.Query(searchQuery, "%"+query+"%", limit)
    if err != nil {
        return nil, err
    }
    defer rows.Close()
    
    var products []Product
    for rows.Next() {
        var p Product
        err := rows.Scan(&p.ID, &p.Name, &p.Description, &p.Price,
            &p.Stock, &p.CategoryID, &p.CreatedAt, &p.UpdatedAt)
        if err != nil {
            return nil, err
        }
        products = append(products, p)
    }
    
    return products, nil
}
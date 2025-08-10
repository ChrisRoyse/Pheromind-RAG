/**
 * REST API controller for user management with Express.js
 * Handles CRUD operations, authentication middleware, and error handling
 */
const express = require('express');
const bcrypt = require('bcrypt');
const jwt = require('jsonwebtoken');
const rateLimit = require('express-rate-limit');

class UserController {
    constructor(database, jwtSecret) {
        this.db = database;
        this.jwtSecret = jwtSecret;
        this.router = express.Router();
        this.setupRoutes();
        this.setupMiddleware();
    }

    setupMiddleware() {
        // Rate limiting for authentication endpoints
        this.authLimiter = rateLimit({
            windowMs: 15 * 60 * 1000, // 15 minutes
            max: 5, // limit each IP to 5 requests per windowMs
            message: 'Too many authentication attempts, try again later'
        });

        // JWT verification middleware
        this.verifyToken = (req, res, next) => {
            const token = req.header('Authorization')?.replace('Bearer ', '');
            if (!token) {
                return res.status(401).json({ error: 'Access denied. No token provided.' });
            }

            try {
                const decoded = jwt.verify(token, this.jwtSecret);
                req.user = decoded;
                next();
            } catch (error) {
                res.status(400).json({ error: 'Invalid token' });
            }
        };
    }

    setupRoutes() {
        this.router.post('/register', this.authLimiter, this.register.bind(this));
        this.router.post('/login', this.authLimiter, this.login.bind(this));
        this.router.get('/profile', this.verifyToken, this.getProfile.bind(this));
        this.router.put('/profile', this.verifyToken, this.updateProfile.bind(this));
        this.router.delete('/account', this.verifyToken, this.deleteAccount.bind(this));
        this.router.post('/refresh-token', this.refreshToken.bind(this));
    }

    async register(req, res) {
        try {
            const { username, email, password } = req.body;
            
            // Validate input
            if (!username || !email || !password) {
                return res.status(400).json({ error: 'Username, email, and password are required' });
            }

            // Check if user already exists
            const existingUser = await this.db.findUserByEmail(email);
            if (existingUser) {
                return res.status(409).json({ error: 'User already exists with this email' });
            }

            // Hash password
            const saltRounds = 12;
            const passwordHash = await bcrypt.hash(password, saltRounds);

            // Create user
            const newUser = await this.db.createUser({
                username,
                email,
                passwordHash,
                createdAt: new Date(),
                isActive: true
            });

            // Generate JWT token
            const token = jwt.sign(
                { userId: newUser.id, username, email },
                this.jwtSecret,
                { expiresIn: '24h' }
            );

            res.status(201).json({
                message: 'User registered successfully',
                token,
                user: { id: newUser.id, username, email }
            });
        } catch (error) {
            console.error('Registration error:', error);
            res.status(500).json({ error: 'Internal server error' });
        }
    }

    async login(req, res) {
        try {
            const { email, password } = req.body;

            if (!email || !password) {
                return res.status(400).json({ error: 'Email and password are required' });
            }

            // Find user
            const user = await this.db.findUserByEmail(email);
            if (!user || !user.isActive) {
                return res.status(401).json({ error: 'Invalid credentials' });
            }

            // Verify password
            const isValidPassword = await bcrypt.compare(password, user.passwordHash);
            if (!isValidPassword) {
                return res.status(401).json({ error: 'Invalid credentials' });
            }

            // Generate JWT token
            const token = jwt.sign(
                { userId: user.id, username: user.username, email: user.email },
                this.jwtSecret,
                { expiresIn: '24h' }
            );

            res.json({
                message: 'Login successful',
                token,
                user: { id: user.id, username: user.username, email: user.email }
            });
        } catch (error) {
            console.error('Login error:', error);
            res.status(500).json({ error: 'Internal server error' });
        }
    }

    async getProfile(req, res) {
        try {
            const user = await this.db.findUserById(req.user.userId);
            if (!user) {
                return res.status(404).json({ error: 'User not found' });
            }

            res.json({
                user: {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    createdAt: user.createdAt,
                    lastLogin: user.lastLogin
                }
            });
        } catch (error) {
            console.error('Profile fetch error:', error);
            res.status(500).json({ error: 'Internal server error' });
        }
    }

    async updateProfile(req, res) {
        try {
            const { username, email } = req.body;
            const userId = req.user.userId;

            const updates = {};
            if (username) updates.username = username;
            if (email) updates.email = email;

            const updatedUser = await this.db.updateUser(userId, updates);
            
            res.json({
                message: 'Profile updated successfully',
                user: {
                    id: updatedUser.id,
                    username: updatedUser.username,
                    email: updatedUser.email
                }
            });
        } catch (error) {
            console.error('Profile update error:', error);
            res.status(500).json({ error: 'Internal server error' });
        }
    }

    async deleteAccount(req, res) {
        try {
            await this.db.deleteUser(req.user.userId);
            res.json({ message: 'Account deleted successfully' });
        } catch (error) {
            console.error('Account deletion error:', error);
            res.status(500).json({ error: 'Internal server error' });
        }
    }

    async refreshToken(req, res) {
        try {
            const { token } = req.body;
            
            if (!token) {
                return res.status(400).json({ error: 'Token is required' });
            }

            const decoded = jwt.verify(token, this.jwtSecret);
            const newToken = jwt.sign(
                { userId: decoded.userId, username: decoded.username, email: decoded.email },
                this.jwtSecret,
                { expiresIn: '24h' }
            );

            res.json({ token: newToken });
        } catch (error) {
            res.status(401).json({ error: 'Invalid or expired token' });
        }
    }
}

module.exports = UserController;
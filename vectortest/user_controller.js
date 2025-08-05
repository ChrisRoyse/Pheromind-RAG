// JavaScript/Node.js User Controller
const express = require('express');
const jwt = require('jsonwebtoken');
const bcrypt = require('bcrypt');

class UserController {
    constructor(userService, emailService) {
        this.userService = userService;
        this.emailService = emailService;
        this.router = express.Router();
        this.setupRoutes();
    }

    setupRoutes() {
        this.router.post('/register', this.register.bind(this));
        this.router.post('/login', this.login.bind(this));
        this.router.get('/profile/:id', this.authenticate, this.getProfile.bind(this));
        this.router.put('/profile/:id', this.authenticate, this.updateProfile.bind(this));
        this.router.delete('/users/:id', this.authenticate, this.authorize('admin'), this.deleteUser.bind(this));
        this.router.get('/users', this.authenticate, this.authorize('admin'), this.listUsers.bind(this));
    }

    async register(req, res) {
        try {
            const { username, email, password } = req.body;
            
            // Validate input
            if (!username || !email || !password) {
                return res.status(400).json({ 
                    error: 'Missing required fields' 
                });
            }

            // Check if user exists
            const existingUser = await this.userService.findByEmail(email);
            if (existingUser) {
                return res.status(409).json({ 
                    error: 'User already exists' 
                });
            }

            // Hash password
            const saltRounds = 10;
            const hashedPassword = await bcrypt.hash(password, saltRounds);

            // Create user
            const user = await this.userService.create({
                username,
                email,
                password: hashedPassword,
                role: 'user',
                isActive: false
            });

            // Send verification email
            const verificationToken = this.generateVerificationToken(user.id);
            await this.emailService.sendVerificationEmail(email, verificationToken);

            res.status(201).json({
                message: 'User created successfully. Please check your email for verification.',
                userId: user.id
            });
        } catch (error) {
            console.error('Registration error:', error);
            res.status(500).json({ 
                error: 'Internal server error' 
            });
        }
    }

    async login(req, res) {
        try {
            const { email, password } = req.body;

            // Find user
            const user = await this.userService.findByEmail(email);
            if (!user) {
                return res.status(401).json({ 
                    error: 'Invalid credentials' 
                });
            }

            // Verify password
            const validPassword = await bcrypt.compare(password, user.password);
            if (!validPassword) {
                return res.status(401).json({ 
                    error: 'Invalid credentials' 
                });
            }

            // Check if user is active
            if (!user.isActive) {
                return res.status(403).json({ 
                    error: 'Please verify your email first' 
                });
            }

            // Generate JWT token
            const token = jwt.sign(
                { 
                    userId: user.id, 
                    email: user.email, 
                    role: user.role 
                },
                process.env.JWT_SECRET,
                { expiresIn: '24h' }
            );

            // Update last login
            await this.userService.updateLastLogin(user.id);

            res.json({
                token,
                user: {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    role: user.role
                }
            });
        } catch (error) {
            console.error('Login error:', error);
            res.status(500).json({ 
                error: 'Internal server error' 
            });
        }
    }

    async getProfile(req, res) {
        try {
            const userId = req.params.id;
            
            // Check if user can access this profile
            if (req.user.userId !== userId && req.user.role !== 'admin') {
                return res.status(403).json({ 
                    error: 'Access denied' 
                });
            }

            const user = await this.userService.findById(userId);
            if (!user) {
                return res.status(404).json({ 
                    error: 'User not found' 
                });
            }

            // Remove sensitive data
            delete user.password;

            res.json(user);
        } catch (error) {
            console.error('Get profile error:', error);
            res.status(500).json({ 
                error: 'Internal server error' 
            });
        }
    }

    async updateProfile(req, res) {
        try {
            const userId = req.params.id;
            const updates = req.body;

            // Check if user can update this profile
            if (req.user.userId !== userId && req.user.role !== 'admin') {
                return res.status(403).json({ 
                    error: 'Access denied' 
                });
            }

            // Prevent updating sensitive fields
            delete updates.password;
            delete updates.role;
            delete updates.isActive;

            const updatedUser = await this.userService.update(userId, updates);
            if (!updatedUser) {
                return res.status(404).json({ 
                    error: 'User not found' 
                });
            }

            res.json({
                message: 'Profile updated successfully',
                user: updatedUser
            });
        } catch (error) {
            console.error('Update profile error:', error);
            res.status(500).json({ 
                error: 'Internal server error' 
            });
        }
    }

    async deleteUser(req, res) {
        try {
            const userId = req.params.id;

            const deleted = await this.userService.delete(userId);
            if (!deleted) {
                return res.status(404).json({ 
                    error: 'User not found' 
                });
            }

            res.json({
                message: 'User deleted successfully'
            });
        } catch (error) {
            console.error('Delete user error:', error);
            res.status(500).json({ 
                error: 'Internal server error' 
            });
        }
    }

    async listUsers(req, res) {
        try {
            const { page = 1, limit = 10, search = '' } = req.query;
            
            const users = await this.userService.list({
                page: parseInt(page),
                limit: parseInt(limit),
                search
            });

            res.json(users);
        } catch (error) {
            console.error('List users error:', error);
            res.status(500).json({ 
                error: 'Internal server error' 
            });
        }
    }

    // Middleware for authentication
    authenticate(req, res, next) {
        const token = req.headers.authorization?.split(' ')[1];

        if (!token) {
            return res.status(401).json({ 
                error: 'Authentication required' 
            });
        }

        try {
            const decoded = jwt.verify(token, process.env.JWT_SECRET);
            req.user = decoded;
            next();
        } catch (error) {
            return res.status(401).json({ 
                error: 'Invalid token' 
            });
        }
    }

    // Middleware for authorization
    authorize(requiredRole) {
        return (req, res, next) => {
            if (req.user.role !== requiredRole) {
                return res.status(403).json({ 
                    error: 'Insufficient permissions' 
                });
            }
            next();
        };
    }

    generateVerificationToken(userId) {
        return jwt.sign(
            { userId, type: 'verification' },
            process.env.JWT_SECRET,
            { expiresIn: '48h' }
        );
    }
}

module.exports = UserController;
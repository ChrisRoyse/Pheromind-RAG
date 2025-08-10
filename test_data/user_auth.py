"""
User authentication service with JWT tokens and bcrypt password hashing.
Implements secure login, registration, and token refresh functionality.
"""
import bcrypt
import jwt
import datetime
from typing import Optional, Dict, Any

class UserAuthService:
    def __init__(self, secret_key: str, token_expiry_hours: int = 24):
        self.secret_key = secret_key
        self.token_expiry_hours = token_expiry_hours
        self.users_db = {}  # In-memory user storage for demo
    
    def hash_password(self, password: str) -> str:
        """Hash password using bcrypt with salt"""
        salt = bcrypt.gensalt()
        return bcrypt.hashpw(password.encode('utf-8'), salt).decode('utf-8')
    
    def verify_password(self, password: str, hashed: str) -> bool:
        """Verify password against hash"""
        return bcrypt.checkpw(password.encode('utf-8'), hashed.encode('utf-8'))
    
    def register_user(self, username: str, password: str, email: str) -> Dict[str, Any]:
        """Register new user with hashed password"""
        if username in self.users_db:
            raise ValueError("User already exists")
        
        hashed_password = self.hash_password(password)
        user_data = {
            'username': username,
            'password_hash': hashed_password,
            'email': email,
            'created_at': datetime.datetime.utcnow(),
            'is_active': True
        }
        self.users_db[username] = user_data
        return {'success': True, 'message': 'User registered successfully'}
    
    def authenticate_user(self, username: str, password: str) -> Optional[str]:
        """Authenticate user and return JWT token"""
        user = self.users_db.get(username)
        if not user or not user['is_active']:
            return None
        
        if not self.verify_password(password, user['password_hash']):
            return None
        
        # Generate JWT token
        payload = {
            'username': username,
            'exp': datetime.datetime.utcnow() + datetime.timedelta(hours=self.token_expiry_hours),
            'iat': datetime.datetime.utcnow()
        }
        return jwt.encode(payload, self.secret_key, algorithm='HS256')
    
    def verify_token(self, token: str) -> Optional[Dict[str, Any]]:
        """Verify JWT token and return user data"""
        try:
            payload = jwt.decode(token, self.secret_key, algorithms=['HS256'])
            return payload
        except jwt.ExpiredSignatureError:
            return None
        except jwt.InvalidTokenError:
            return None
    
    def refresh_token(self, token: str) -> Optional[str]:
        """Refresh JWT token if valid"""
        payload = self.verify_token(token)
        if not payload:
            return None
        
        new_payload = {
            'username': payload['username'],
            'exp': datetime.datetime.utcnow() + datetime.timedelta(hours=self.token_expiry_hours),
            'iat': datetime.datetime.utcnow()
        }
        return jwt.encode(new_payload, self.secret_key, algorithm='HS256')
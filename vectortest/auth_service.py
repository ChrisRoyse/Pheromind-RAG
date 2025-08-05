# Python Authentication Service
import hashlib
import secrets
from datetime import datetime, timedelta
from typing import Optional, Dict, List

class AuthenticationService:
    """Handles user authentication and session management"""
    
    def __init__(self):
        self.users: Dict[str, Dict] = {}
        self.sessions: Dict[str, Dict] = {}
        self.token_expiry = timedelta(hours=24)
    
    def hash_password(self, password: str, salt: Optional[str] = None) -> tuple[str, str]:
        """Hash a password with salt using SHA-256"""
        if not salt:
            salt = secrets.token_hex(16)
        
        password_hash = hashlib.sha256(f"{password}{salt}".encode()).hexdigest()
        return password_hash, salt
    
    def create_user(self, username: str, email: str, password: str) -> bool:
        """Create a new user account"""
        if username in self.users:
            raise ValueError(f"User {username} already exists")
        
        password_hash, salt = self.hash_password(password)
        
        self.users[username] = {
            'email': email,
            'password_hash': password_hash,
            'salt': salt,
            'created_at': datetime.now(),
            'last_login': None,
            'is_active': True
        }
        return True
    
    def authenticate(self, username: str, password: str) -> Optional[str]:
        """Authenticate user and return session token"""
        if username not in self.users:
            return None
        
        user = self.users[username]
        password_hash, _ = self.hash_password(password, user['salt'])
        
        if password_hash != user['password_hash']:
            return None
        
        # Generate session token
        token = secrets.token_urlsafe(32)
        self.sessions[token] = {
            'username': username,
            'created_at': datetime.now(),
            'expires_at': datetime.now() + self.token_expiry
        }
        
        # Update last login
        user['last_login'] = datetime.now()
        
        return token
    
    def validate_token(self, token: str) -> Optional[str]:
        """Validate session token and return username"""
        if token not in self.sessions:
            return None
        
        session = self.sessions[token]
        if datetime.now() > session['expires_at']:
            del self.sessions[token]
            return None
        
        return session['username']
    
    def logout(self, token: str) -> bool:
        """Logout user by removing session"""
        if token in self.sessions:
            del self.sessions[token]
            return True
        return False
    
    def change_password(self, username: str, old_password: str, new_password: str) -> bool:
        """Change user password"""
        if not self.authenticate(username, old_password):
            return False
        
        password_hash, salt = self.hash_password(new_password)
        self.users[username]['password_hash'] = password_hash
        self.users[username]['salt'] = salt
        
        return True
    
    def list_active_sessions(self) -> List[Dict]:
        """List all active sessions"""
        active_sessions = []
        current_time = datetime.now()
        
        for token, session in list(self.sessions.items()):
            if current_time > session['expires_at']:
                del self.sessions[token]
            else:
                active_sessions.append({
                    'username': session['username'],
                    'created_at': session['created_at'],
                    'expires_at': session['expires_at']
                })
        
        return active_sessions
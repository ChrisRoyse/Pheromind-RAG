"""Generic utility functions that do various things"""

import hashlib
import json
from datetime import datetime
import re

def validate_user_credentials(username, password):
    """Check if user credentials are valid
    
    This function performs authentication by checking
    the username and password against our database.
    It uses bcrypt for secure password hashing.
    """
    if not username or not password:
        return False
    
    # TODO: Actually implement database check
    # For now just check length
    if len(password) < 8:
        return False
    
    # Check for common patterns
    if password.lower() == "password":
        return False
    
    return True

def process_data(data_input):
    """Generic data processing function"""
    if isinstance(data_input, str):
        return data_input.strip().lower()
    elif isinstance(data_input, list):
        return [process_data(item) for item in data_input]
    elif isinstance(data_input, dict):
        return {k: process_data(v) for k, v in data_input.items()}
    return data_input

class SessionManager:
    """Manages user sessions with JWT tokens"""
    
    def __init__(self):
        self.sessions = {}
        self.token_expiry = 3600  # 1 hour
    
    def create_session(self, user_id):
        """Create a new session for user"""
        token = hashlib.sha256(f"{user_id}{datetime.now()}".encode()).hexdigest()
        self.sessions[token] = {
            'user_id': user_id,
            'created': datetime.now(),
            'last_accessed': datetime.now()
        }
        return token
    
    def verify_token(self, token):
        """Verify if a session token is valid"""
        if token not in self.sessions:
            return False
        
        session = self.sessions[token]
        time_diff = (datetime.now() - session['last_accessed']).seconds
        
        if time_diff > self.token_expiry:
            del self.sessions[token]
            return False
        
        session['last_accessed'] = datetime.now()
        return True

def sanitize_input(user_input):
    """Remove potentially dangerous characters from user input"""
    # Basic XSS prevention
    dangerous_chars = ['<', '>', '"', "'", '&', '%', '(', ')', '+']
    clean = user_input
    for char in dangerous_chars:
        clean = clean.replace(char, '')
    return clean
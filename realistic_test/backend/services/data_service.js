// Main service for handling data operations
const axios = require('axios');
const cache = new Map();

class DataService {
    constructor(apiUrl) {
        this.apiUrl = apiUrl;
        this.retryCount = 3;
        this.timeout = 5000;
    }

    async fetchUserData(userId) {
        // Check cache first
        const cacheKey = `user_${userId}`;
        if (cache.has(cacheKey)) {
            const cached = cache.get(cacheKey);
            if (Date.now() - cached.timestamp < 60000) {
                return cached.data;
            }
        }

        try {
            const response = await axios.get(`${this.apiUrl}/users/${userId}`, {
                timeout: this.timeout
            });
            
            // Store in cache
            cache.set(cacheKey, {
                data: response.data,
                timestamp: Date.now()
            });
            
            return response.data;
        } catch (error) {
            console.error('Failed to fetch user:', error);
            throw new Error('User fetch failed');
        }
    }

    async updateUserProfile(userId, profileData) {
        // Validate profile data
        if (!profileData.name || !profileData.email) {
            throw new Error('Invalid profile data');
        }

        const updates = {
            ...profileData,
            updatedAt: new Date().toISOString()
        };

        return await this.makeRequest('PUT', `/users/${userId}`, updates);
    }

    async searchUsers(query, filters = {}) {
        const params = new URLSearchParams();
        params.append('q', query);
        
        Object.keys(filters).forEach(key => {
            params.append(key, filters[key]);
        });

        const response = await axios.get(`${this.apiUrl}/users/search?${params}`);
        return response.data;
    }

    async makeRequest(method, endpoint, data = null) {
        let attempts = 0;
        
        while (attempts < this.retryCount) {
            try {
                const config = {
                    method,
                    url: `${this.apiUrl}${endpoint}`,
                    timeout: this.timeout
                };
                
                if (data) {
                    config.data = data;
                }
                
                const response = await axios(config);
                return response.data;
            } catch (error) {
                attempts++;
                if (attempts >= this.retryCount) {
                    throw error;
                }
                await this.delay(1000 * attempts);
            }
        }
    }

    delay(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    clearCache() {
        cache.clear();
    }
}

module.exports = DataService;
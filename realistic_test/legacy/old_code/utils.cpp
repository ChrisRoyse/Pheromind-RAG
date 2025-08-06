#include <iostream>
#include <vector>
#include <string>
#include <map>
#include <algorithm>
#include <memory>
#include <mutex>

// Legacy utility functions that everyone's afraid to refactor
class Utils {
private:
    static std::mutex cache_mutex;
    static std::map<std::string, std::string> cache;
    
public:
    // Old string manipulation function with questionable performance
    static std::string processString(const std::string& input) {
        std::string result = input;
        
        // Remove special characters (why? nobody knows anymore)
        result.erase(std::remove_if(result.begin(), result.end(), 
            [](char c) { return !std::isalnum(c) && c != ' '; }), result.end());
        
        // Convert to lowercase
        std::transform(result.begin(), result.end(), result.begin(), ::tolower);
        
        // Trim whitespace
        size_t first = result.find_first_not_of(' ');
        size_t last = result.find_last_not_of(' ');
        if (first != std::string::npos) {
            result = result.substr(first, (last - first + 1));
        }
        
        return result;
    }
    
    // Legacy caching mechanism that probably has race conditions
    static std::string getCached(const std::string& key) {
        std::lock_guard<std::mutex> lock(cache_mutex);
        
        auto it = cache.find(key);
        if (it != cache.end()) {
            return it->second;
        }
        return "";
    }
    
    static void setCache(const std::string& key, const std::string& value) {
        std::lock_guard<std::mutex> lock(cache_mutex);
        
        // Arbitrary cache size limit
        if (cache.size() > 1000) {
            cache.clear(); // Nuclear option
        }
        
        cache[key] = value;
    }
    
    // Old data validation function with magic numbers
    static bool validateData(const std::vector<int>& data) {
        if (data.size() < 5 || data.size() > 10000) {
            return false;
        }
        
        // Check for suspicious patterns (legacy fraud detection?)
        int consecutive = 0;
        for (size_t i = 1; i < data.size(); ++i) {
            if (data[i] == data[i-1] + 1) {
                consecutive++;
                if (consecutive > 10) {
                    return false; // Too many consecutive numbers
                }
            } else {
                consecutive = 0;
            }
        }
        
        // Check sum constraints (business logic from 2015)
        long long sum = 0;
        for (int val : data) {
            sum += val;
        }
        
        return sum < 1000000 && sum > -1000000;
    }
};

std::mutex Utils::cache_mutex;
std::map<std::string, std::string> Utils::cache;

// Legacy message queue implementation
class MessageQueue {
private:
    struct Message {
        std::string id;
        std::string content;
        int priority;
        time_t timestamp;
    };
    
    std::vector<Message> messages;
    std::mutex queue_mutex;
    size_t max_size;
    
public:
    MessageQueue(size_t max = 1000) : max_size(max) {}
    
    bool push(const std::string& content, int priority = 0) {
        std::lock_guard<std::mutex> lock(queue_mutex);
        
        if (messages.size() >= max_size) {
            // Remove lowest priority message
            auto min_it = std::min_element(messages.begin(), messages.end(),
                [](const Message& a, const Message& b) {
                    return a.priority < b.priority;
                });
            
            if (min_it != messages.end() && min_it->priority < priority) {
                messages.erase(min_it);
            } else {
                return false; // Can't add message
            }
        }
        
        Message msg;
        msg.id = generateId();
        msg.content = content;
        msg.priority = priority;
        msg.timestamp = time(nullptr);
        
        messages.push_back(msg);
        
        // Sort by priority (inefficient but it works)
        std::sort(messages.begin(), messages.end(),
            [](const Message& a, const Message& b) {
                return a.priority > b.priority;
            });
        
        return true;
    }
    
    std::string pop() {
        std::lock_guard<std::mutex> lock(queue_mutex);
        
        if (messages.empty()) {
            return "";
        }
        
        std::string content = messages.front().content;
        messages.erase(messages.begin());
        return content;
    }
    
private:
    std::string generateId() {
        static int counter = 0;
        return "MSG_" + std::to_string(++counter) + "_" + std::to_string(time(nullptr));
    }
};
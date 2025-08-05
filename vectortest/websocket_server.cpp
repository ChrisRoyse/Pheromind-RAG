// C++ WebSocket Server Implementation
#include <iostream>
#include <memory>
#include <string>
#include <thread>
#include <unordered_map>
#include <unordered_set>
#include <queue>
#include <mutex>
#include <condition_variable>
#include <chrono>
#include <functional>
#include <websocketpp/config/asio.hpp>
#include <websocketpp/server.hpp>
#include <json/json.h>
#include <boost/uuid/uuid.hpp>
#include <boost/uuid/uuid_generators.hpp>
#include <boost/uuid/uuid_io.hpp>

typedef websocketpp::server<websocketpp::config::asio> WebSocketServer;
typedef WebSocketServer::message_ptr message_ptr;

class MessageBroadcaster;
class ConnectionManager;
class RateLimiter;

// Custom connection data
struct ConnectionData {
    std::string id;
    std::string userId;
    std::unordered_set<std::string> subscribedChannels;
    std::chrono::steady_clock::time_point lastActivity;
    bool authenticated;
};

// Message types
enum class MessageType {
    CONNECT,
    DISCONNECT,
    AUTHENTICATE,
    SUBSCRIBE,
    UNSUBSCRIBE,
    MESSAGE,
    BROADCAST,
    PING,
    PONG,
    ERROR
};

// Main WebSocket server class
class RealtimeServer {
private:
    WebSocketServer server;
    std::unique_ptr<ConnectionManager> connectionManager;
    std::unique_ptr<MessageBroadcaster> broadcaster;
    std::unique_ptr<RateLimiter> rateLimiter;
    
    std::thread broadcastThread;
    std::thread cleanupThread;
    
    std::atomic<bool> running{false};
    
public:
    RealtimeServer() {
        // Initialize server
        server.set_error_channels(websocketpp::log::elevel::all);
        server.set_access_channels(websocketpp::log::alevel::all ^ websocketpp::log::alevel::frame_payload);
        
        server.init_asio();
        
        // Initialize components
        connectionManager = std::make_unique<ConnectionManager>();
        broadcaster = std::make_unique<MessageBroadcaster>(&server, connectionManager.get());
        rateLimiter = std::make_unique<RateLimiter>();
        
        // Set handlers
        server.set_open_handler(std::bind(&RealtimeServer::onOpen, this, std::placeholders::_1));
        server.set_close_handler(std::bind(&RealtimeServer::onClose, this, std::placeholders::_1));
        server.set_message_handler(std::bind(&RealtimeServer::onMessage, this, std::placeholders::_1, std::placeholders::_2));
        server.set_fail_handler(std::bind(&RealtimeServer::onFail, this, std::placeholders::_1));
        
        // Start background threads
        running = true;
        broadcastThread = std::thread(&MessageBroadcaster::run, broadcaster.get());
        cleanupThread = std::thread(&RealtimeServer::cleanupInactiveConnections, this);
    }
    
    ~RealtimeServer() {
        stop();
    }
    
    void start(uint16_t port) {
        server.set_reuse_addr(true);
        server.listen(port);
        server.start_accept();
        
        std::cout << "WebSocket server started on port " << port << std::endl;
        
        server.run();
    }
    
    void stop() {
        running = false;
        
        server.stop_listening();
        server.stop();
        
        if (broadcastThread.joinable()) {
            broadcastThread.join();
        }
        if (cleanupThread.joinable()) {
            cleanupThread.join();
        }
    }
    
private:
    void onOpen(websocketpp::connection_hdl hdl) {
        auto conn = server.get_con_from_hdl(hdl);
        std::string connectionId = generateUUID();
        
        // Create connection data
        auto data = std::make_shared<ConnectionData>();
        data->id = connectionId;
        data->lastActivity = std::chrono::steady_clock::now();
        data->authenticated = false;
        
        connectionManager->addConnection(hdl, data);
        
        // Send welcome message
        Json::Value welcome;
        welcome["type"] = "welcome";
        welcome["connectionId"] = connectionId;
        welcome["timestamp"] = getCurrentTimestamp();
        
        sendMessage(hdl, welcome);
        
        std::cout << "New connection: " << connectionId << std::endl;
    }
    
    void onClose(websocketpp::connection_hdl hdl) {
        auto data = connectionManager->getConnectionData(hdl);
        if (data) {
            std::cout << "Connection closed: " << data->id << std::endl;
            
            // Unsubscribe from all channels
            for (const auto& channel : data->subscribedChannels) {
                connectionManager->unsubscribeFromChannel(hdl, channel);
            }
        }
        
        connectionManager->removeConnection(hdl);
    }
    
    void onMessage(websocketpp::connection_hdl hdl, message_ptr msg) {
        auto data = connectionManager->getConnectionData(hdl);
        if (!data) {
            return;
        }
        
        // Update last activity
        data->lastActivity = std::chrono::steady_clock::now();
        
        // Rate limiting
        if (!rateLimiter->allowRequest(data->id)) {
            sendError(hdl, "RATE_LIMIT_EXCEEDED", "Too many requests");
            return;
        }
        
        try {
            Json::Value message;
            Json::Reader reader;
            
            if (!reader.parse(msg->get_payload(), message)) {
                sendError(hdl, "INVALID_JSON", "Failed to parse JSON");
                return;
            }
            
            handleMessage(hdl, data, message);
            
        } catch (const std::exception& e) {
            sendError(hdl, "PROCESSING_ERROR", e.what());
        }
    }
    
    void onFail(websocketpp::connection_hdl hdl) {
        auto conn = server.get_con_from_hdl(hdl);
        std::cout << "Connection failed: " << conn->get_ec().message() << std::endl;
    }
    
    void handleMessage(websocketpp::connection_hdl hdl, 
                      std::shared_ptr<ConnectionData> data,
                      const Json::Value& message) {
        
        std::string type = message.get("type", "").asString();
        
        if (type == "authenticate") {
            handleAuthentication(hdl, data, message);
        }
        else if (type == "subscribe") {
            handleSubscribe(hdl, data, message);
        }
        else if (type == "unsubscribe") {
            handleUnsubscribe(hdl, data, message);
        }
        else if (type == "message") {
            handleUserMessage(hdl, data, message);
        }
        else if (type == "broadcast") {
            handleBroadcast(hdl, data, message);
        }
        else if (type == "ping") {
            handlePing(hdl, data);
        }
        else {
            sendError(hdl, "UNKNOWN_MESSAGE_TYPE", "Unknown message type: " + type);
        }
    }
    
    void handleAuthentication(websocketpp::connection_hdl hdl,
                            std::shared_ptr<ConnectionData> data,
                            const Json::Value& message) {
        std::string token = message.get("token", "").asString();
        
        // Validate token (simplified - in production, verify JWT or session)
        if (token.empty()) {
            sendError(hdl, "INVALID_TOKEN", "Authentication token is required");
            return;
        }
        
        // Extract user ID from token (simplified)
        data->userId = "user_" + token.substr(0, 8);
        data->authenticated = true;
        
        Json::Value response;
        response["type"] = "authenticated";
        response["userId"] = data->userId;
        response["timestamp"] = getCurrentTimestamp();
        
        sendMessage(hdl, response);
        
        std::cout << "Client authenticated: " << data->id << " as " << data->userId << std::endl;
    }
    
    void handleSubscribe(websocketpp::connection_hdl hdl,
                        std::shared_ptr<ConnectionData> data,
                        const Json::Value& message) {
        if (!data->authenticated) {
            sendError(hdl, "NOT_AUTHENTICATED", "Authentication required");
            return;
        }
        
        std::string channel = message.get("channel", "").asString();
        if (channel.empty()) {
            sendError(hdl, "INVALID_CHANNEL", "Channel name is required");
            return;
        }
        
        // Check channel permissions (simplified)
        if (!hasChannelAccess(data->userId, channel)) {
            sendError(hdl, "ACCESS_DENIED", "No access to channel: " + channel);
            return;
        }
        
        connectionManager->subscribeToChannel(hdl, channel);
        data->subscribedChannels.insert(channel);
        
        Json::Value response;
        response["type"] = "subscribed";
        response["channel"] = channel;
        response["timestamp"] = getCurrentTimestamp();
        
        sendMessage(hdl, response);
        
        std::cout << "Client " << data->id << " subscribed to " << channel << std::endl;
    }
    
    void handleUnsubscribe(websocketpp::connection_hdl hdl,
                          std::shared_ptr<ConnectionData> data,
                          const Json::Value& message) {
        std::string channel = message.get("channel", "").asString();
        if (channel.empty()) {
            sendError(hdl, "INVALID_CHANNEL", "Channel name is required");
            return;
        }
        
        connectionManager->unsubscribeFromChannel(hdl, channel);
        data->subscribedChannels.erase(channel);
        
        Json::Value response;
        response["type"] = "unsubscribed";
        response["channel"] = channel;
        response["timestamp"] = getCurrentTimestamp();
        
        sendMessage(hdl, response);
        
        std::cout << "Client " << data->id << " unsubscribed from " << channel << std::endl;
    }
    
    void handleUserMessage(websocketpp::connection_hdl hdl,
                          std::shared_ptr<ConnectionData> data,
                          const Json::Value& message) {
        if (!data->authenticated) {
            sendError(hdl, "NOT_AUTHENTICATED", "Authentication required");
            return;
        }
        
        std::string channel = message.get("channel", "").asString();
        Json::Value content = message.get("data", Json::Value());
        
        if (channel.empty()) {
            sendError(hdl, "INVALID_CHANNEL", "Channel is required");
            return;
        }
        
        // Check if user is subscribed to channel
        if (data->subscribedChannels.find(channel) == data->subscribedChannels.end()) {
            sendError(hdl, "NOT_SUBSCRIBED", "Not subscribed to channel: " + channel);
            return;
        }
        
        // Prepare message for broadcast
        Json::Value broadcastMsg;
        broadcastMsg["type"] = "message";
        broadcastMsg["channel"] = channel;
        broadcastMsg["userId"] = data->userId;
        broadcastMsg["data"] = content;
        broadcastMsg["timestamp"] = getCurrentTimestamp();
        
        broadcaster->queueMessage(channel, broadcastMsg);
    }
    
    void handleBroadcast(websocketpp::connection_hdl hdl,
                        std::shared_ptr<ConnectionData> data,
                        const Json::Value& message) {
        if (!data->authenticated) {
            sendError(hdl, "NOT_AUTHENTICATED", "Authentication required");
            return;
        }
        
        // Check if user has broadcast permissions (simplified)
        if (!hasBroadcastPermission(data->userId)) {
            sendError(hdl, "ACCESS_DENIED", "No broadcast permission");
            return;
        }
        
        Json::Value content = message.get("data", Json::Value());
        
        // Broadcast to all connected clients
        Json::Value broadcastMsg;
        broadcastMsg["type"] = "broadcast";
        broadcastMsg["userId"] = data->userId;
        broadcastMsg["data"] = content;
        broadcastMsg["timestamp"] = getCurrentTimestamp();
        
        broadcaster->queueGlobalBroadcast(broadcastMsg);
    }
    
    void handlePing(websocketpp::connection_hdl hdl,
                   std::shared_ptr<ConnectionData> data) {
        Json::Value pong;
        pong["type"] = "pong";
        pong["timestamp"] = getCurrentTimestamp();
        
        sendMessage(hdl, pong);
    }
    
    void sendMessage(websocketpp::connection_hdl hdl, const Json::Value& message) {
        try {
            Json::FastWriter writer;
            std::string payload = writer.write(message);
            
            server.send(hdl, payload, websocketpp::frame::opcode::text);
        } catch (const websocketpp::exception& e) {
            std::cerr << "Failed to send message: " << e.what() << std::endl;
        }
    }
    
    void sendError(websocketpp::connection_hdl hdl, 
                  const std::string& code,
                  const std::string& message) {
        Json::Value error;
        error["type"] = "error";
        error["error"]["code"] = code;
        error["error"]["message"] = message;
        error["timestamp"] = getCurrentTimestamp();
        
        sendMessage(hdl, error);
    }
    
    void cleanupInactiveConnections() {
        while (running) {
            std::this_thread::sleep_for(std::chrono::seconds(30));
            
            auto now = std::chrono::steady_clock::now();
            auto inactiveThreshold = std::chrono::minutes(5);
            
            connectionManager->removeInactiveConnections(now, inactiveThreshold);
        }
    }
    
    bool hasChannelAccess(const std::string& userId, const std::string& channel) {
        // Simplified permission check
        // In production, check against database or permission service
        return true;
    }
    
    bool hasBroadcastPermission(const std::string& userId) {
        // Simplified permission check
        // In production, check user roles and permissions
        return userId.find("admin") != std::string::npos;
    }
    
    std::string generateUUID() {
        boost::uuids::random_generator gen;
        boost::uuids::uuid uuid = gen();
        return boost::uuids::to_string(uuid);
    }
    
    int64_t getCurrentTimestamp() {
        return std::chrono::duration_cast<std::chrono::milliseconds>(
            std::chrono::system_clock::now().time_since_epoch()
        ).count();
    }
};

// Connection manager
class ConnectionManager {
private:
    mutable std::mutex mutex;
    std::unordered_map<websocketpp::connection_hdl, 
                      std::shared_ptr<ConnectionData>,
                      std::owner_less<websocketpp::connection_hdl>> connections;
    std::unordered_map<std::string, 
                      std::unordered_set<websocketpp::connection_hdl,
                                       std::owner_less<websocketpp::connection_hdl>>> channels;
    
public:
    void addConnection(websocketpp::connection_hdl hdl, 
                      std::shared_ptr<ConnectionData> data) {
        std::lock_guard<std::mutex> lock(mutex);
        connections[hdl] = data;
    }
    
    void removeConnection(websocketpp::connection_hdl hdl) {
        std::lock_guard<std::mutex> lock(mutex);
        connections.erase(hdl);
    }
    
    std::shared_ptr<ConnectionData> getConnectionData(websocketpp::connection_hdl hdl) const {
        std::lock_guard<std::mutex> lock(mutex);
        auto it = connections.find(hdl);
        return it != connections.end() ? it->second : nullptr;
    }
    
    void subscribeToChannel(websocketpp::connection_hdl hdl, const std::string& channel) {
        std::lock_guard<std::mutex> lock(mutex);
        channels[channel].insert(hdl);
    }
    
    void unsubscribeFromChannel(websocketpp::connection_hdl hdl, const std::string& channel) {
        std::lock_guard<std::mutex> lock(mutex);
        auto it = channels.find(channel);
        if (it != channels.end()) {
            it->second.erase(hdl);
            if (it->second.empty()) {
                channels.erase(it);
            }
        }
    }
    
    std::unordered_set<websocketpp::connection_hdl, std::owner_less<websocketpp::connection_hdl>> 
    getChannelSubscribers(const std::string& channel) const {
        std::lock_guard<std::mutex> lock(mutex);
        auto it = channels.find(channel);
        return it != channels.end() ? it->second : std::unordered_set<websocketpp::connection_hdl, std::owner_less<websocketpp::connection_hdl>>();
    }
    
    std::vector<websocketpp::connection_hdl> getAllConnections() const {
        std::lock_guard<std::mutex> lock(mutex);
        std::vector<websocketpp::connection_hdl> result;
        for (const auto& pair : connections) {
            result.push_back(pair.first);
        }
        return result;
    }
    
    void removeInactiveConnections(std::chrono::steady_clock::time_point now,
                                 std::chrono::steady_clock::duration threshold) {
        std::lock_guard<std::mutex> lock(mutex);
        
        auto it = connections.begin();
        while (it != connections.end()) {
            if (now - it->second->lastActivity > threshold) {
                // Remove from channels
                for (const auto& channel : it->second->subscribedChannels) {
                    auto channelIt = channels.find(channel);
                    if (channelIt != channels.end()) {
                        channelIt->second.erase(it->first);
                    }
                }
                
                std::cout << "Removing inactive connection: " << it->second->id << std::endl;
                it = connections.erase(it);
            } else {
                ++it;
            }
        }
    }
};

// Message broadcaster
class MessageBroadcaster {
private:
    WebSocketServer* server;
    ConnectionManager* connectionManager;
    
    mutable std::mutex queueMutex;
    std::condition_variable cv;
    std::queue<std::pair<std::string, Json::Value>> messageQueue;
    std::atomic<bool> running{true};
    
public:
    MessageBroadcaster(WebSocketServer* srv, ConnectionManager* connMgr)
        : server(srv), connectionManager(connMgr) {}
    
    void queueMessage(const std::string& channel, const Json::Value& message) {
        {
            std::lock_guard<std::mutex> lock(queueMutex);
            messageQueue.push({channel, message});
        }
        cv.notify_one();
    }
    
    void queueGlobalBroadcast(const Json::Value& message) {
        queueMessage("*", message);
    }
    
    void run() {
        while (running) {
            std::unique_lock<std::mutex> lock(queueMutex);
            cv.wait(lock, [this] { return !messageQueue.empty() || !running; });
            
            while (!messageQueue.empty()) {
                auto [channel, message] = messageQueue.front();
                messageQueue.pop();
                lock.unlock();
                
                broadcastMessage(channel, message);
                
                lock.lock();
            }
        }
    }
    
    void stop() {
        running = false;
        cv.notify_all();
    }
    
private:
    void broadcastMessage(const std::string& channel, const Json::Value& message) {
        std::vector<websocketpp::connection_hdl> recipients;
        
        if (channel == "*") {
            // Global broadcast
            recipients = connectionManager->getAllConnections();
        } else {
            // Channel broadcast
            auto subscribers = connectionManager->getChannelSubscribers(channel);
            recipients.assign(subscribers.begin(), subscribers.end());
        }
        
        Json::FastWriter writer;
        std::string payload = writer.write(message);
        
        for (const auto& hdl : recipients) {
            try {
                server->send(hdl, payload, websocketpp::frame::opcode::text);
            } catch (const websocketpp::exception& e) {
                std::cerr << "Broadcast failed: " << e.what() << std::endl;
            }
        }
    }
};

// Rate limiter
class RateLimiter {
private:
    mutable std::mutex mutex;
    std::unordered_map<std::string, std::queue<std::chrono::steady_clock::time_point>> requests;
    
    size_t maxRequests = 100;
    std::chrono::seconds window{60};
    
public:
    bool allowRequest(const std::string& clientId) {
        std::lock_guard<std::mutex> lock(mutex);
        
        auto now = std::chrono::steady_clock::now();
        auto& clientRequests = requests[clientId];
        
        // Remove old requests outside the window
        while (!clientRequests.empty() && 
               now - clientRequests.front() > window) {
            clientRequests.pop();
        }
        
        if (clientRequests.size() >= maxRequests) {
            return false;
        }
        
        clientRequests.push(now);
        return true;
    }
};

// Main function
int main() {
    try {
        RealtimeServer server;
        server.start(9002);
    } catch (const std::exception& e) {
        std::cerr << "Server error: " << e.what() << std::endl;
        return 1;
    }
    
    return 0;
}
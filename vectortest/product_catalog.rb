# Ruby Product Catalog Service
require 'active_record'
require 'elasticsearch'
require 'redis'

class ProductCatalog
  include ActiveModel::Model
  
  attr_accessor :elasticsearch_client, :redis_client, :logger
  
  def initialize
    @elasticsearch_client = Elasticsearch::Client.new(
      host: ENV['ELASTICSEARCH_HOST'] || 'localhost:9200',
      log: true
    )
    
    @redis_client = Redis.new(
      host: ENV['REDIS_HOST'] || 'localhost',
      port: ENV['REDIS_PORT'] || 6379
    )
    
    @logger = Rails.logger || Logger.new(STDOUT)
  end
  
  # Search products with various filters
  def search(query:, filters: {}, page: 1, per_page: 20)
    cache_key = generate_cache_key(query, filters, page, per_page)
    
    # Check cache first
    cached_result = get_from_cache(cache_key)
    return cached_result if cached_result
    
    # Build Elasticsearch query
    search_body = build_search_query(query, filters, page, per_page)
    
    begin
      response = @elasticsearch_client.search(
        index: 'products',
        body: search_body
      )
      
      result = parse_search_response(response)
      
      # Cache the result
      set_cache(cache_key, result, expires_in: 5.minutes)
      
      result
    rescue Elasticsearch::Transport::Transport::Errors::NotFound => e
      @logger.error "Elasticsearch index not found: #{e.message}"
      { products: [], total: 0, facets: {} }
    rescue StandardError => e
      @logger.error "Search error: #{e.message}"
      raise SearchError, "Failed to search products"
    end
  end
  
  # Index a product in Elasticsearch
  def index_product(product)
    document = {
      id: product.id,
      name: product.name,
      description: product.description,
      category: product.category.name,
      category_id: product.category_id,
      brand: product.brand,
      price: product.price,
      sale_price: product.sale_price,
      in_stock: product.in_stock?,
      inventory_count: product.inventory_count,
      tags: product.tags.pluck(:name),
      attributes: product.attributes_hash,
      created_at: product.created_at,
      updated_at: product.updated_at,
      rating: product.average_rating,
      review_count: product.reviews.count
    }
    
    @elasticsearch_client.index(
      index: 'products',
      id: product.id,
      body: document
    )
    
    # Invalidate related caches
    invalidate_product_caches(product)
  end
  
  # Bulk index products
  def bulk_index(products)
    return if products.empty?
    
    body = products.flat_map do |product|
      [
        { index: { _index: 'products', _id: product.id } },
        build_product_document(product)
      ]
    end
    
    @elasticsearch_client.bulk(body: body)
    
    @logger.info "Bulk indexed #{products.count} products"
  end
  
  # Get product recommendations
  def get_recommendations(product_id:, limit: 10)
    product = Product.find(product_id)
    
    # Get similar products based on category and attributes
    similar_products = find_similar_products(product, limit: limit * 2)
    
    # Get frequently bought together products
    frequently_bought = get_frequently_bought_together(product_id, limit: limit)
    
    # Combine and deduplicate recommendations
    recommendations = (similar_products + frequently_bought).uniq(&:id).take(limit)
    
    # Score and sort recommendations
    scored_recommendations = score_recommendations(recommendations, product)
    
    scored_recommendations.sort_by { |r| -r[:score] }
  end
  
  # Get trending products
  def get_trending(category: nil, timeframe: 24.hours, limit: 20)
    cache_key = "trending_products:#{category}:#{timeframe}:#{limit}"
    
    cached_result = get_from_cache(cache_key)
    return cached_result if cached_result
    
    # Calculate trending score based on views, purchases, and recency
    trending_products = Product
      .joins(:product_views, :order_items)
      .where('product_views.created_at > ?', timeframe.ago)
      .group('products.id')
      .select(
        'products.*',
        'COUNT(DISTINCT product_views.id) as view_count',
        'COUNT(DISTINCT order_items.id) as purchase_count'
      )
    
    trending_products = trending_products.where(category: category) if category
    
    # Calculate trending score
    products_with_scores = trending_products.map do |product|
      score = calculate_trending_score(
        view_count: product.view_count,
        purchase_count: product.purchase_count,
        recency_factor: calculate_recency_factor(product.created_at)
      )
      
      { product: product, score: score }
    end
    
    # Sort by score and limit
    result = products_with_scores
      .sort_by { |item| -item[:score] }
      .take(limit)
      .map { |item| item[:product] }
    
    set_cache(cache_key, result, expires_in: 1.hour)
    
    result
  end
  
  # Autocomplete suggestions
  def autocomplete(query:, limit: 10)
    return [] if query.blank? || query.length < 2
    
    cache_key = "autocomplete:#{query.downcase}:#{limit}"
    cached_result = get_from_cache(cache_key)
    return cached_result if cached_result
    
    response = @elasticsearch_client.search(
      index: 'products',
      body: {
        query: {
          multi_match: {
            query: query,
            type: 'phrase_prefix',
            fields: ['name^3', 'brand^2', 'category']
          }
        },
        _source: ['name', 'brand', 'category'],
        size: limit
      }
    )
    
    suggestions = response['hits']['hits'].map do |hit|
      {
        text: hit['_source']['name'],
        category: hit['_source']['category'],
        brand: hit['_source']['brand']
      }
    end
    
    set_cache(cache_key, suggestions, expires_in: 10.minutes)
    
    suggestions
  end
  
  private
  
  def build_search_query(query, filters, page, per_page)
    must_clauses = []
    filter_clauses = []
    
    # Text search
    if query.present?
      must_clauses << {
        multi_match: {
          query: query,
          fields: ['name^3', 'description^2', 'brand^2', 'category', 'tags'],
          type: 'best_fields',
          operator: 'and'
        }
      }
    end
    
    # Apply filters
    filter_clauses << { term: { category_id: filters[:category_id] } } if filters[:category_id]
    filter_clauses << { term: { brand: filters[:brand] } } if filters[:brand]
    filter_clauses << { range: { price: { gte: filters[:min_price], lte: filters[:max_price] } } } if filters[:min_price] || filters[:max_price]
    filter_clauses << { term: { in_stock: true } } if filters[:in_stock]
    
    # Rating filter
    if filters[:min_rating]
      filter_clauses << { range: { rating: { gte: filters[:min_rating] } } }
    end
    
    {
      query: {
        bool: {
          must: must_clauses,
          filter: filter_clauses
        }
      },
      aggs: build_aggregations,
      from: (page - 1) * per_page,
      size: per_page,
      sort: build_sort_clause(filters[:sort_by])
    }
  end
  
  def build_aggregations
    {
      categories: { terms: { field: 'category_id', size: 20 } },
      brands: { terms: { field: 'brand.keyword', size: 20 } },
      price_ranges: {
        range: {
          field: 'price',
          ranges: [
            { to: 25 },
            { from: 25, to: 50 },
            { from: 50, to: 100 },
            { from: 100, to: 200 },
            { from: 200 }
          ]
        }
      },
      avg_rating: { avg: { field: 'rating' } }
    }
  end
  
  def build_sort_clause(sort_by)
    case sort_by
    when 'price_asc'
      [{ price: 'asc' }]
    when 'price_desc'
      [{ price: 'desc' }]
    when 'rating'
      [{ rating: 'desc' }, { review_count: 'desc' }]
    when 'newest'
      [{ created_at: 'desc' }]
    else
      ['_score']
    end
  end
  
  def parse_search_response(response)
    products = response['hits']['hits'].map do |hit|
      Product.find(hit['_id'])
    end
    
    {
      products: products,
      total: response['hits']['total']['value'],
      facets: parse_aggregations(response['aggregations'])
    }
  end
  
  def parse_aggregations(aggregations)
    return {} unless aggregations
    
    {
      categories: aggregations['categories']['buckets'],
      brands: aggregations['brands']['buckets'],
      price_ranges: aggregations['price_ranges']['buckets'],
      avg_rating: aggregations['avg_rating']['value']
    }
  end
  
  def find_similar_products(product, limit:)
    Product
      .where(category_id: product.category_id)
      .where.not(id: product.id)
      .where(in_stock: true)
      .order(rating: :desc, review_count: :desc)
      .limit(limit)
  end
  
  def get_frequently_bought_together(product_id, limit:)
    order_item_ids = OrderItem.where(product_id: product_id).pluck(:order_id)
    
    Product
      .joins(:order_items)
      .where(order_items: { order_id: order_item_ids })
      .where.not(id: product_id)
      .group('products.id')
      .order('COUNT(order_items.id) DESC')
      .limit(limit)
  end
  
  def calculate_trending_score(view_count:, purchase_count:, recency_factor:)
    # Weighted score: purchases are worth more than views
    base_score = (view_count * 1) + (purchase_count * 10)
    base_score * recency_factor
  end
  
  def calculate_recency_factor(created_at)
    # Exponential decay based on age
    days_old = (Time.current - created_at) / 1.day
    Math.exp(-0.1 * days_old)
  end
  
  def score_recommendations(products, reference_product)
    products.map do |product|
      score = 0
      
      # Category match
      score += 10 if product.category_id == reference_product.category_id
      
      # Price similarity (within 20%)
      price_diff = (product.price - reference_product.price).abs / reference_product.price
      score += 5 if price_diff <= 0.2
      
      # Rating
      score += product.rating * 2
      
      # Popularity (review count)
      score += Math.log(product.review_count + 1)
      
      { product: product, score: score }
    end
  end
  
  def generate_cache_key(query, filters, page, per_page)
    key_parts = ["product_search", query, filters.to_json, page, per_page]
    Digest::MD5.hexdigest(key_parts.join(':'))
  end
  
  def get_from_cache(key)
    cached = @redis_client.get(key)
    return nil unless cached
    
    JSON.parse(cached, symbolize_names: true)
  rescue StandardError => e
    @logger.warn "Cache read error: #{e.message}"
    nil
  end
  
  def set_cache(key, value, expires_in:)
    @redis_client.setex(key, expires_in.to_i, value.to_json)
  rescue StandardError => e
    @logger.warn "Cache write error: #{e.message}"
  end
  
  def invalidate_product_caches(product)
    # Invalidate all caches that might contain this product
    pattern = "product_search:*"
    keys = @redis_client.keys(pattern)
    @redis_client.del(*keys) unless keys.empty?
  end
  
  def build_product_document(product)
    {
      id: product.id,
      name: product.name,
      description: product.description,
      category: product.category.name,
      category_id: product.category_id,
      brand: product.brand,
      price: product.price,
      sale_price: product.sale_price,
      in_stock: product.in_stock?,
      inventory_count: product.inventory_count,
      tags: product.tags.pluck(:name),
      attributes: product.attributes_hash,
      created_at: product.created_at,
      updated_at: product.updated_at,
      rating: product.average_rating,
      review_count: product.reviews.count
    }
  end
end

class SearchError < StandardError; end
# Table-Heavy Documentation

This document contains extensive table examples to test table preservation during chunking.

## Simple Data Tables

### User Information

| ID | Name | Email | Age | Department |
|----|------|-------|-----|------------|
| 1  | Alice Johnson | alice@company.com | 28 | Engineering |
| 2  | Bob Smith | bob@company.com | 35 | Marketing |
| 3  | Carol White | carol@company.com | 42 | Sales |
| 4  | David Brown | david@company.com | 31 | Engineering |
| 5  | Eve Davis | eve@company.com | 29 | HR |

### Product Catalog

| SKU | Product Name | Category | Price | Stock | Supplier |
|-----|-------------|----------|-------|-------|----------|
| ABC123 | Laptop Pro | Electronics | $1,299.99 | 45 | TechSupply |
| DEF456 | Office Chair | Furniture | $299.99 | 23 | ComfortSeating |
| GHI789 | Coffee Maker | Appliances | $89.99 | 67 | KitchenPlus |
| JKL012 | Monitor 27" | Electronics | $349.99 | 12 | DisplayCorp |
| MNO345 | Desk Lamp | Lighting | $59.99 | 89 | LightSolutions |

## Complex Tables with Formatting

### Performance Metrics

| Metric | Q1 2023 | Q2 2023 | Q3 2023 | Q4 2023 | YoY Change | Target |
|--------|---------|---------|---------|---------|------------|--------|
| **Revenue** | $2.1M | $2.4M | $2.8M | $3.1M | +15.2% | ✅ $3.0M |
| **Users** | 45,230 | 52,100 | 61,450 | 68,900 | +24.1% | ✅ 65,000 |
| **Conversion** | 3.2% | 3.8% | 4.1% | 4.5% | +0.8pp | ✅ 4.0% |
| **Churn Rate** | 2.1% | 1.9% | 1.7% | 1.5% | -0.4pp | ✅ 1.8% |
| **ARPU** | $46.41 | $46.07 | $45.55 | $44.99 | -$1.42 | ❌ $47.00 |
| **CAC** | $89.50 | $85.20 | $82.10 | $79.30 | -$8.90 | ✅ $80.00 |

### API Endpoint Documentation

| Endpoint | Method | Auth Required | Rate Limit | Response Format | Description |
|----------|--------|---------------|------------|-----------------|-------------|
| `/api/users` | GET | ✅ Bearer | 1000/hour | JSON | List all users |
| `/api/users/{id}` | GET | ✅ Bearer | 1000/hour | JSON | Get user by ID |
| `/api/users` | POST | ✅ Bearer | 100/hour | JSON | Create new user |
| `/api/users/{id}` | PUT | ✅ Bearer | 100/hour | JSON | Update user |
| `/api/users/{id}` | DELETE | ✅ Bearer | 50/hour | 204 No Content | Delete user |
| `/api/auth/login` | POST | ❌ None | 10/minute | JSON | User login |
| `/api/auth/refresh` | POST | ✅ Refresh | 20/hour | JSON | Refresh token |
| `/api/posts` | GET | ❌ None | 5000/hour | JSON | List public posts |
| `/api/posts/{id}` | GET | ❌ None | 5000/hour | JSON | Get post by ID |
| `/api/posts` | POST | ✅ Bearer | 50/hour | JSON | Create new post |

## Tables with Special Characters and Unicode

### International Data

| Country | Capital | Currency | Population | GDP (USD) | Official Language(s) |
|---------|---------|----------|------------|-----------|---------------------|
| 🇺🇸 USA | Washington D.C. | $ Dollar | 331.9M | $23.32T | English |
| 🇩🇪 Germany | Berlin | € Euro | 83.2M | $4.26T | Deutsch |
| 🇯🇵 Japan | Tokyo | ¥ Yen | 125.8M | $4.94T | 日本語 |
| 🇨🇳 China | Beijing | ¥ Yuan | 1.41B | $17.73T | 中文 |
| 🇮🇳 India | New Delhi | ₹ Rupee | 1.38B | $3.39T | हिन्दी, English |
| 🇧🇷 Brazil | Brasília | R$ Real | 215.3M | $2.05T | Português |
| 🇷🇺 Russia | Moscow | ₽ Ruble | 144.1M | $1.78T | Русский |
| 🇫🇷 France | Paris | € Euro | 67.4M | $2.94T | Français |

### Mathematical Constants

| Symbol | Name | Value | Formula | Application |
|--------|------|-------|---------|-------------|
| π | Pi | 3.14159... | C/d | Circle circumference |
| e | Euler's number | 2.71828... | lim(1+1/n)ⁿ | Natural logarithm base |
| φ | Golden ratio | 1.61803... | (1+√5)/2 | Art, architecture |
| √2 | Square root of 2 | 1.41421... | √2 | Diagonal of unit square |
| γ | Euler-Mascheroni | 0.57721... | lim(Σ1/k - ln(n)) | Number theory |
| ∞ | Infinity | ∞ | ∞ | Limits, calculus |

## Nested Tables and Complex Structures

### Project Timeline

| Phase | Task | Owner | Start Date | End Date | Status | Dependencies | Notes |
|-------|------|-------|------------|----------|--------|--------------|--------|
| **Phase 1: Planning** | | | | | | | |
| 1.1 | Requirements Analysis | PM Team | 2024-01-01 | 2024-01-15 | ✅ Complete | - | Stakeholder interviews done |
| 1.2 | Technical Design | Architects | 2024-01-10 | 2024-01-25 | ✅ Complete | 1.1 | Architecture approved |
| 1.3 | Resource Planning | PM Team | 2024-01-20 | 2024-01-30 | ✅ Complete | 1.2 | Team assignments finalized |
| **Phase 2: Development** | | | | | | | |
| 2.1 | Backend API | Dev Team A | 2024-02-01 | 2024-03-15 | 🚧 In Progress | 1.3 | 65% complete |
| 2.2 | Frontend UI | Dev Team B | 2024-02-15 | 2024-03-30 | 🚧 In Progress | 1.3, 2.1 | 40% complete |
| 2.3 | Database Setup | DevOps | 2024-02-01 | 2024-02-15 | ✅ Complete | 1.3 | Production ready |
| **Phase 3: Testing** | | | | | | | |
| 3.1 | Unit Testing | Dev Teams | 2024-03-01 | 2024-03-20 | 🚧 In Progress | 2.1, 2.2 | 30% complete |
| 3.2 | Integration Testing | QA Team | 2024-03-20 | 2024-04-05 | ⏸️ Pending | 3.1 | Waiting for dev completion |
| 3.3 | User Acceptance | Business | 2024-04-05 | 2024-04-20 | ⏸️ Pending | 3.2 | UAT environment ready |

### Budget Breakdown

| Category | Subcategory | Budget | Actual | Remaining | % Used | Variance | Notes |
|----------|-------------|--------|--------|-----------|--------|----------|--------|
| **Personnel** | | $500,000 | $345,000 | $155,000 | 69% | Under budget | |
| | Developers | $300,000 | $210,000 | $90,000 | 70% | $-90,000 | 2 contractors added |
| | QA Engineers | $100,000 | $75,000 | $25,000 | 75% | $-25,000 | On track |
| | Project Managers | $100,000 | $60,000 | $40,000 | 60% | $-40,000 | Part-time PM |
| **Infrastructure** | | $200,000 | $180,000 | $20,000 | 90% | Over budget | |
| | Cloud Services | $120,000 | $130,000 | $-10,000 | 108% | $+10,000 | Higher than expected usage |
| | Monitoring Tools | $50,000 | $35,000 | $15,000 | 70% | $-15,000 | Negotiated discount |
| | Security Services | $30,000 | $15,000 | $15,000 | 50% | $-15,000 | Delayed implementation |
| **Software/Licenses** | | $100,000 | $85,000 | $15,000 | 85% | Under budget | |
| | Development Tools | $60,000 | $55,000 | $5,000 | 92% | $-5,000 | Volume discount received |
| | Third-party APIs | $40,000 | $30,000 | $10,000 | 75% | $-10,000 | Lower usage than projected |

## Comparison Tables

### Technology Stack Comparison

| Feature | Option A: React | Option B: Vue.js | Option C: Angular | Recommendation |
|---------|----------------|------------------|-------------------|----------------|
| **Learning Curve** | Medium | Easy | Hard | Vue.js wins |
| **Performance** | Excellent | Excellent | Good | Tie: React/Vue |
| **Community** | Huge | Large | Large | React wins |
| **Job Market** | Excellent | Good | Excellent | Tie: React/Angular |
| **Bundle Size** | Medium | Small | Large | Vue.js wins |
| **TypeScript** | Good | Good | Excellent | Angular wins |
| **Ecosystem** | Rich | Growing | Comprehensive | React wins |
| **Enterprise Support** | Good | Limited | Excellent | Angular wins |
| **Developer Experience** | Excellent | Excellent | Good | Tie: React/Vue |
| **Testing Tools** | Mature | Mature | Excellent | Angular wins |
| **State Management** | Redux/Context | Vuex/Pinia | NgRx | Context dependent |
| **Mobile Development** | React Native | NativeScript | Ionic | React wins |

**Final Score:**
- React: 4 wins, 3 ties
- Vue.js: 3 wins, 3 ties  
- Angular: 4 wins, 1 tie

### Cloud Provider Comparison

| Service Category | AWS | Azure | Google Cloud | Oracle | IBM Cloud |
|------------------|-----|-------|--------------|--------|-----------|
| **Compute** | | | | | |
| Virtual Machines | EC2 | VM | Compute Engine | Compute | VSI |
| Containers | ECS/EKS | ACI/AKS | GKE | OKE | IKS |
| Serverless | Lambda | Functions | Cloud Functions | Fn | Functions |
| **Storage** | | | | | |
| Object Storage | S3 | Blob Storage | Cloud Storage | Object Storage | Object Storage |
| Block Storage | EBS | Disk Storage | Persistent Disk | Block Volume | Block Storage |
| File Storage | EFS | File Storage | Filestore | File Storage | File Storage |
| **Database** | | | | | |
| Relational | RDS | SQL Database | Cloud SQL | Database | Db2/PostgreSQL |
| NoSQL | DynamoDB | Cosmos DB | Firestore | NoSQL | Cloudant |
| Data Warehouse | Redshift | Synapse | BigQuery | Autonomous DW | Db2 Warehouse |
| **Pricing** | | | | | |
| Free Tier | 12 months | 12 months | Always free | 30 days | Lite plans |
| Pay-as-you-go | ✅ | ✅ | ✅ | ✅ | ✅ |
| Reserved Instances | ✅ | ✅ | ✅ | ✅ | Limited |
| **Global Presence** | | | | | |
| Regions | 32 | 60+ | 35 | 44 | 19 |
| Edge Locations | 410+ | 165+ | 146 | N/A | Limited |

## Data Analysis Tables

### Sales Performance by Region

| Region | Q1 Sales | Q2 Sales | Q3 Sales | Q4 Sales | Total | Growth | Top Product |
|--------|----------|----------|----------|----------|-------|--------|-------------|
| **North America** | $1.2M | $1.4M | $1.6M | $1.8M | **$6.0M** | +15.2% | Laptop Pro |
| East Coast | $400K | $450K | $520K | $580K | $1.95M | +12.8% | Monitor 27" |
| West Coast | $500K | $600K | $650K | $750K | $2.5M | +18.9% | Laptop Pro |
| Central | $300K | $350K | $430K | $470K | $1.55M | +14.1% | Office Chair |
| **Europe** | $800K | $900K | $1.1M | $1.3M | **$4.1M** | +22.5% | Coffee Maker |
| UK | $250K | $280K | $350K | $420K | $1.3M | +19.8% | Desk Lamp |
| Germany | $200K | $220K | $290K | $350K | $1.06M | +24.2% | Coffee Maker |
| France | $180K | $200K | $240K | $280K | $900K | +16.7% | Monitor 27" |
| Other EU | $170K | $200K | $220K | $250K | $840K | +13.8% | Office Chair |
| **Asia Pacific** | $600K | $750K | $900K | $1.1M | **$3.35M** | +28.4% | Laptop Pro |
| Japan | $200K | $250K | $300K | $380K | $1.13M | +25.6% | Monitor 27" |
| Australia | $150K | $180K | $220K | $270K | $820K | +22.1% | Laptop Pro |
| Singapore | $120K | $150K | $190K | $230K | $690K | +29.3% | Coffee Maker |
| Other APAC | $130K | $170K | $190K | $220K | $710K | +19.4% | Desk Lamp |

### Customer Satisfaction Survey Results

| Aspect | Excellent | Good | Average | Poor | Very Poor | Satisfaction Score | Trend |
|---------|-----------|------|---------|------|-----------|--------------------|--------|
| **Product Quality** | 45% | 35% | 15% | 4% | 1% | 4.19/5 | ↗️ +0.15 |
| **Customer Service** | 38% | 42% | 16% | 3% | 1% | 4.13/5 | ↗️ +0.08 |
| **Delivery Speed** | 52% | 32% | 12% | 3% | 1% | 4.31/5 | ↗️ +0.22 |
| **Website Usability** | 41% | 39% | 17% | 2% | 1% | 4.17/5 | ↘️ -0.05 |
| **Value for Money** | 33% | 44% | 18% | 4% | 1% | 4.04/5 | ↗️ +0.11 |
| **Return Process** | 28% | 38% | 24% | 8% | 2% | 3.82/5 | ↘️ -0.03 |
| **Product Selection** | 47% | 36% | 14% | 2% | 1% | 4.26/5 | ↗️ +0.18 |
| **Mobile App** | 35% | 41% | 19% | 4% | 1% | 4.05/5 | ↗️ +0.09 |

**Overall Satisfaction:** 4.12/5 (↗️ +0.07 from last quarter)

This document tests table handling in various scenarios:
- Simple data tables with basic formatting
- Complex tables with mixed content types
- Tables with special characters and Unicode
- Nested tables and hierarchical data
- Comparison matrices
- Performance and analytics tables
- Multi-level categorization in tables
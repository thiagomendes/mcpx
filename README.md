# mcpx

**MCP eXtended Gateway — Production-ready SaaS for managing MCP servers at scale**

## 🎯 What is mcpx?

**mcpx (MCP eXtended Gateway)** is a production-ready SaaS platform for managing Model Context Protocol (MCP) servers at scale.

It bridges the gap between prototype and production by solving the hard problems: **multi-server orchestration**, **tool-level governance**, **secure credential management**, **complete observability**, and **team collaboration** — without requiring infrastructure expertise.

---

## 📖 Table of Contents

- [What is mcpx?](#-what-is-mcpx)
- [Key Features](#-key-features)
  - [Connect Instantly](#-connect-instantly)
  - [Governance & Security](#️-governance--security)
  - [Observability](#-observability)
  - [Virtual Gateways](#-virtual-gateways)
  - [Developer Experience](#-developer-experience)
- [Quick Start](#-quick-start)
- [Use Cases](#-use-cases)
- [Why mcpx?](#-why-mcpx)
- [Documentation](#-documentation)
- [Design Philosophy](#-design-philosophy)
- [Who Uses mcpx?](#-who-uses-mcpx)
- [Get in Touch](#-get-in-touch)

---

## ✨ Key Features

### 🔌 **Connect Instantly**
Point to your existing MCP servers via URL—no deployment, no Docker, no infrastructure required. Just paste a URL and you're live.

### 🛡️ **Governance & Security**
- **Tool-level control:** Whitelist/blacklist specific AI tools (e.g., allow `get_weather`, block `delete_database`)
- **OAuth authentication:** Sign in with Google (Microsoft Azure AD coming soon)
- **Fine-grained permissions:** Control who can access which servers

### 📊 **Observability**
- **Audit logs:** Complete request history with timestamps, tool names, and response times
- **Analytics dashboard:** Track usage patterns, identify bottlenecks, monitor costs
- **Real-time monitoring:** See active requests and server health

### 🌐 **Virtual Gateways**
Aggregate multiple MCP servers into a single endpoint. Connect Claude Desktop to one gateway URL and access 10+ servers seamlessly.

### ⚡ **Developer Experience**
- **No CLI required:** Manage everything through an intuitive web dashboard
- **Instant updates:** Changes take effect immediately, no restarts needed
- **API-first:** Programmatic access for automation and CI/CD integration

## 🚀 Quick Start

### 1. **Sign In**
Visit the web dashboard and sign in with your Google account.

### 2. **Connect Your First MCP Server**
Click **"Add Server"** and enter:
- **Name:** My Weather Server
- **URL:** `https://mcp.example.com`
- **Auth:** (optional) Bearer token or API key

### 3. **Configure Claude Desktop**
Copy your server's proxy URL and paste it into Claude Desktop's MCP config:
```json
{
  "mcpServers": {
    "my-weather": {
      "url": "https://mcpx.app/proxy/your-user-id/my-weather"
    }
  }
}
```

### 4. **Start Using AI Tools**
Ask Claude: *"What's the weather in San Francisco?"* — mcpx proxies the request, logs it, and returns the response.

---

## 🎓 Use Cases

### For Developers
- **Rapid prototyping:** Connect experimental MCP servers without deployment overhead
- **Testing & debugging:** Inspect every MCP request/response in the dashboard
- **Multi-environment management:** Separate dev, staging, and production servers

### For Product Teams
- **Tool governance:** Control which AI capabilities are exposed to end users
- **Usage analytics:** Understand which tools are most valuable
- **Cost monitoring:** Track MCP request volume and optimize

### For Enterprises
- **Audit compliance:** Complete request history for security reviews
- **Access control:** Restrict sensitive tools to authorized teams
- **SSO integration:** Google OAuth today, Azure AD and SAML coming soon

## 🌟 Why mcpx?

### ❌ **Without mcpx:**
- Deploy and manage MCP servers yourself (Docker, K8s, cloud infra)
- Write custom auth and governance logic for every server
- Build your own logging and monitoring dashboards
- Manually aggregate multiple servers for Claude Desktop
- Debug production issues without visibility

### ✅ **With mcpx:**
- **10-second setup:** Paste a URL, start using AI tools immediately
- **Built-in governance:** Tool-level control out of the box
- **Complete observability:** Every request logged and searchable
- **Virtual gateways:** One endpoint for all your MCP servers
- **Production-ready:** Authentication, monitoring, and reliability handled for you

---

## 📖 Documentation

### For End Users
- [Quick Start Guide](./docs/SPEC.md#getting-started) - Connect your first server in 5 minutes
- [User Guide](./docs/SPEC.md#core-features) - All features explained with screenshots
- [Use Cases](./docs/SPEC.md#market-opportunity) - Real-world examples

### For Developers & Contributors
- [Technical Specification](./docs/SPEC.md) - Complete product architecture
- [API Documentation](./docs/SPEC.md#rest-api-endpoints) - Programmatic access
- [Implementation Roadmap](./docs/SPEC.md#implementation-roadmap) - Development plan

## 🎨 Design Philosophy

**Modern, Accessible, Fast**
- Clean, intuitive interface inspired by modern SaaS dashboards (Vercel, Railway)
- Dark theme optimized for developers and long working sessions
- Responsive design - works on desktop, tablet, and mobile
- Accessible (WCAG 2.1 AA compliant)

**Visual Identity by TM Dev Lab:**
- Primary: Indigo (#6366f1) + Pink (#ec4899) gradient accents
- Background: Dark slate (#020617) with elevated cards (#1e293b)
- Typography: Inter for UI, JetBrains Mono for code

---

## 📊 Who Uses mcpx?

### 👨‍💻 **Developers**
Build and test MCP servers without deployment complexity. Iterate quickly with instant feedback.

### 🚀 **Startups**
Ship AI features faster by focusing on product, not infrastructure. Scale without hiring DevOps.

### 🏢 **Enterprises**
Meet compliance requirements with audit logs and access control. Integrate with existing SSO.

### 🎓 **Educators**
Teach AI development without requiring students to manage cloud infrastructure.

### 🎨 **Product Teams**
Control which AI capabilities reach users. Monitor usage to inform roadmap decisions.

---

## 📞 Get in Touch

**Questions or feedback?** mail.thiagomendes@gmail.com

**Owner:** TM Dev Lab

---

## 🚀 Get Started

Ready to simplify your MCP workflow? Check out the [Technical Specification](./docs/SPEC.md) to see how mcpx works under the hood, or dive straight into the [Quick Start Guide](./docs/SPEC.md#getting-started).

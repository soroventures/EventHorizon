import React, { useState } from 'react'
import { Book, Code, Cpu, Server, Map, FileText } from 'lucide-react'

export function Docs() {
  const [activeSection, setActiveSection] = useState('overview')

  const sections = [
    { id: 'overview', icon: Book, title: 'Overview' },
    { id: 'getting-started', icon: FileText, title: 'Getting Started' },
    { id: 'architecture', icon: Cpu, title: 'Architecture' },
    { id: 'core-features', icon: Code, title: 'Core Features' },
    { id: 'deployment', icon: Server, title: 'Deployment Notes' },
    { id: 'roadmap', icon: Map, title: 'Roadmap' },
  ]

  const content = {
    'overview': (
      <div className="space-y-4 animate-in">
        <h2 className="text-3xl font-bold tracking-tight">Overview</h2>
        <p className="text-muted-foreground text-lg">
          EventHorizon is a high-performance event listener protocol for blockchain networks, specifically designed for Stellar and Soroban smart contracts.
        </p>
        <p className="text-muted-foreground">
          It provides a seamless bridge between on-chain events and off-chain infrastructure, enabling developers to build responsive, event-driven applications without maintaining complex polling logic.
        </p>
      </div>
    ),
    'getting-started': (
      <div className="space-y-4 animate-in">
        <h2 className="text-3xl font-bold tracking-tight">Getting Started</h2>
        <p className="text-muted-foreground">
          To begin listening to contract events, navigate to the Dashboard and create a new Trigger.
        </p>
        <div className="bg-black/50 p-4 rounded-lg font-mono text-sm border border-border mt-4">
          <span className="text-purple-400">POST</span> /api/v1/triggers
          <br /><br />
          {`{
  "contractId": "CDLZ...",
  "eventName": "transfer",
  "webhookUrl": "https://api.yourdomain.com/webhook"
}`}
        </div>
      </div>
    ),
    'architecture': (
      <div className="space-y-4 animate-in">
        <h2 className="text-3xl font-bold tracking-tight">Architecture</h2>
        <p className="text-muted-foreground">
          The system is split into two primary components: the <strong>API Server</strong> and the <strong>Worker Nodes</strong>.
        </p>
        <ul className="list-disc pl-6 space-y-2 text-muted-foreground mt-4">
          <li><strong>API Server:</strong> Handles user requests, authentication, and trigger management. Built with Node.js and Express.</li>
          <li><strong>Worker Nodes:</strong> Distributed polling engines that continuously scan the blockchain ledger for emitted events matching active triggers.</li>
          <li><strong>Message Queue:</strong> Redis-backed queue connecting the workers to the webhook dispatcher to ensure reliable delivery.</li>
        </ul>
      </div>
    ),
    'core-features': (
      <div className="space-y-4 animate-in">
        <h2 className="text-3xl font-bold tracking-tight">Core Features</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mt-6">
          <div className="p-4 border border-border rounded-lg bg-secondary/20">
            <h4 className="font-semibold mb-2">Sub-second Latency</h4>
            <p className="text-sm text-muted-foreground">Workers are optimized to detect events within the same ledger close.</p>
          </div>
          <div className="p-4 border border-border rounded-lg bg-secondary/20">
            <h4 className="font-semibold mb-2">Guaranteed Delivery</h4>
            <p className="text-sm text-muted-foreground">Built-in retry mechanisms with exponential backoff for failing webhooks.</p>
          </div>
          <div className="p-4 border border-border rounded-lg bg-secondary/20">
            <h4 className="font-semibold mb-2">Signature Verification</h4>
            <p className="text-sm text-muted-foreground">HMAC signatures on all webhook payloads to verify origin authenticity.</p>
          </div>
        </div>
      </div>
    ),
    'deployment': (
      <div className="space-y-4 animate-in">
        <h2 className="text-3xl font-bold tracking-tight">Deployment Notes</h2>
        <p className="text-muted-foreground">
          For production deployments, it is highly recommended to run the worker nodes separately from the API server.
        </p>
        <div className="bg-black/50 p-4 rounded-lg font-mono text-sm border border-border mt-4 text-emerald-400">
          docker-compose -f docker-compose.prod.yml up -d
        </div>
      </div>
    ),
    'roadmap': (
      <div className="space-y-4 animate-in">
        <h2 className="text-3xl font-bold tracking-tight">Roadmap</h2>
        <ul className="space-y-4 mt-4">
          <li className="flex items-center gap-3 text-muted-foreground">
            <div className="h-2 w-2 rounded-full bg-emerald-500"></div>
            <span>Stellar Classic asset payment triggers</span>
          </li>
          <li className="flex items-center gap-3 text-muted-foreground">
            <div className="h-2 w-2 rounded-full bg-emerald-500"></div>
            <span>Soroban smart contract event parsing</span>
          </li>
          <li className="flex items-center gap-3 text-muted-foreground">
            <div className="h-2 w-2 rounded-full bg-indigo-500 animate-pulse"></div>
            <span className="text-foreground">Discord & Telegram native integrations</span>
          </li>
          <li className="flex items-center gap-3 text-muted-foreground opacity-50">
            <div className="h-2 w-2 rounded-full bg-border"></div>
            <span>GraphQL Subscription support</span>
          </li>
        </ul>
      </div>
    )
  }

  return (
    <div className="container max-w-screen-2xl py-10">
      <div className="flex flex-col md:flex-row gap-8">
        
        {/* Sidebar */}
        <aside className="w-full md:w-64 shrink-0">
          <div className="sticky top-24">
            <h3 className="font-semibold mb-4 text-sm uppercase tracking-wider text-muted-foreground">Documentation</h3>
            <nav className="space-y-1">
              {sections.map(section => (
                <button
                  key={section.id}
                  onClick={() => setActiveSection(section.id)}
                  className={`w-full flex items-center gap-3 px-3 py-2 text-sm rounded-md transition-colors ${
                    activeSection === section.id 
                    ? 'bg-primary/10 text-primary font-medium' 
                    : 'text-muted-foreground hover:bg-secondary/50 hover:text-foreground'
                  }`}
                >
                  <section.icon className="h-4 w-4" />
                  {section.title}
                </button>
              ))}
            </nav>
          </div>
        </aside>

        {/* Content */}
        <main className="flex-1 min-h-[500px] bg-secondary/10 border border-border rounded-xl p-8 lg:p-12">
          {content[activeSection]}
        </main>
      </div>
    </div>
  )
}

import React, { useState } from 'react'
import { motion } from 'framer-motion'
import { Bell, Shield, Terminal, Zap, Plus, Trash2, CheckCircle2, Clock } from 'lucide-react'
import { Button } from '../components/ui/Button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../components/ui/Card'

export function Dashboard() {
  const [contractId, setContractId] = useState('')
  const [eventName, setEventName] = useState('')
  const [webhookUrl, setWebhookUrl] = useState('')
  
  // Mock data for realism
  const [triggers] = useState([
    { id: 1, contract: 'C...', event: 'transfer', status: 'active', calls: 142 },
    { id: 2, contract: 'C...', event: 'mint', status: 'active', calls: 56 }
  ])

  const [logs] = useState([
    { id: 1, time: '14:23:45', msg: 'Worker node initialized on cluster-us-east' },
    { id: 2, time: '14:24:12', msg: 'Subscribed to events for contract C...' },
    { id: 3, time: '14:25:01', msg: 'Event matching trigger #1 fired. Executing webhook.' },
    { id: 4, time: '14:25:01', msg: 'Webhook delivery successful (200 OK).' }
  ])

  return (
    <div className="container max-w-screen-2xl py-10">
      <div className="flex flex-col md:flex-row gap-4 justify-between items-start mb-8">
        <div>
          <h1 className="text-3xl font-bold tracking-tight mb-2">Dashboard</h1>
          <p className="text-muted-foreground">Manage your event triggers and monitor real-time node activity.</p>
        </div>
        <Button variant="gradient" className="gap-2">
          <Zap className="h-4 w-4" /> Connect Wallet
        </Button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Creation Form */}
        <motion.div 
          initial={{ opacity: 0, x: -20 }}
          animate={{ opacity: 1, x: 0 }}
          className="lg:col-span-1"
        >
          <Card className="glass sticky top-24">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Bell size={20} className="text-indigo-400" />
                New Trigger
              </CardTitle>
              <CardDescription>Configure a new smart contract event listener.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <label className="text-xs font-medium text-muted-foreground uppercase tracking-wider">Contract ID</label>
                <input 
                  type="text" 
                  value={contractId}
                  onChange={(e) => setContractId(e.target.value)}
                  placeholder="e.g. CDLZ..." 
                  className="w-full bg-background border border-input rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all"
                />
              </div>
              <div className="space-y-2">
                <label className="text-xs font-medium text-muted-foreground uppercase tracking-wider">Event Name</label>
                <input 
                  type="text" 
                  value={eventName}
                  onChange={(e) => setEventName(e.target.value)}
                  placeholder="e.g. transfer" 
                  className="w-full bg-background border border-input rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all"
                />
              </div>
              <div className="space-y-2">
                <label className="text-xs font-medium text-muted-foreground uppercase tracking-wider">Webhook URL</label>
                <input 
                  type="url" 
                  value={webhookUrl}
                  onChange={(e) => setWebhookUrl(e.target.value)}
                  placeholder="https://your-api.com/webhook" 
                  className="w-full bg-background border border-input rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all"
                />
              </div>
              <Button className="w-full mt-4 gap-2">
                <Plus className="h-4 w-4" /> Create Listener
              </Button>
            </CardContent>
          </Card>
        </motion.div>

        {/* Status Section */}
        <motion.div 
          initial={{ opacity: 0, x: 20 }}
          animate={{ opacity: 1, x: 0 }}
          className="lg:col-span-2 space-y-6"
        >
          <Card className="glass">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Shield size={20} className="text-emerald-400" />
                Active Listeners
              </CardTitle>
            </CardHeader>
            <CardContent>
              {triggers.length > 0 ? (
                <div className="space-y-3">
                  {triggers.map(trigger => (
                    <div key={trigger.id} className="flex items-center justify-between p-4 rounded-lg bg-secondary/30 border border-border hover:bg-secondary/50 transition-colors">
                      <div className="flex items-center gap-4">
                        <div className="h-2 w-2 rounded-full bg-emerald-500 animate-pulse" />
                        <div>
                          <p className="font-medium text-sm">{trigger.event} <span className="text-muted-foreground ml-2 font-mono">{trigger.contract}</span></p>
                          <p className="text-xs text-muted-foreground mt-1 flex items-center gap-1">
                            <CheckCircle2 className="h-3 w-3" /> {trigger.calls} successful calls
                          </p>
                        </div>
                      </div>
                      <Button variant="ghost" size="icon" className="text-destructive hover:bg-destructive/20">
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="text-muted-foreground text-sm flex flex-col items-center justify-center py-12 border-2 border-dashed border-border rounded-xl">
                  <Shield size={32} className="mb-4 text-muted-foreground/50" />
                  <p>No active triggers yet.</p>
                  <p className="text-xs mt-1">Create one using the form to get started.</p>
                </div>
              )}
            </CardContent>
          </Card>

          <Card className="glass">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Terminal size={20} className="text-indigo-400" />
                Live Node Feed
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="bg-black/80 p-4 rounded-xl font-mono text-xs text-indigo-300 h-64 overflow-y-auto border border-border/50">
                {logs.map(log => (
                  <div key={log.id} className="mb-2 flex items-start gap-3 opacity-80 hover:opacity-100 transition-opacity">
                    <span className="text-muted-foreground shrink-0 flex items-center gap-1">
                      <Clock className="h-3 w-3" /> {log.time}
                    </span>
                    <span className="break-all">{log.msg}</span>
                  </div>
                ))}
                <div className="animate-pulse text-muted-foreground mt-4">_ Waiting for events...</div>
              </div>
            </CardContent>
          </Card>
        </motion.div>
      </div>
    </div>
  )
}

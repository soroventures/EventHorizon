import React from 'react'
import { motion } from 'framer-motion'
import { ArrowRight, Activity, Shield, Zap } from 'lucide-react'
import { Button } from '../components/ui/Button'
import { Link } from 'react-router-dom'

export function Home() {
  const containerVariants = {
    hidden: { opacity: 0 },
    visible: {
      opacity: 1,
      transition: { staggerChildren: 0.1 }
    }
  }

  const itemVariants = {
    hidden: { opacity: 0, y: 20 },
    visible: {
      opacity: 1,
      y: 0,
      transition: { duration: 0.5, ease: "easeOut" }
    }
  }

  return (
    <div className="flex flex-col min-h-[calc(100vh-4rem)]">
      {/* Hero Section */}
      <section className="relative flex-1 flex flex-col items-center justify-center overflow-hidden py-24 lg:py-32">
        <div className="absolute inset-0 -z-10 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-indigo-900/20 via-background to-background"></div>
        
        <div className="container px-4 md:px-6 flex flex-col items-center text-center space-y-8 max-w-screen-2xl">
          <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ duration: 0.8, ease: "easeOut" }}
            className="space-y-4 max-w-[800px]"
          >
            <h1 className="text-4xl md:text-6xl lg:text-7xl font-bold tracking-tighter sm:text-5xl">
              The Ultimate Web3 <br />
              <span className="text-gradient-primary">Event Listener Protocol</span>
            </h1>
            <p className="mx-auto max-w-[600px] text-muted-foreground md:text-xl/relaxed lg:text-base/relaxed xl:text-xl/relaxed">
              Real-time monitoring, deep integrations, and instant alerts for smart contract events. Built for the modern decentralized web.
            </p>
          </motion.div>
          
          <motion.div 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3, duration: 0.5 }}
            className="flex flex-col sm:flex-row gap-4"
          >
            <Button size="lg" variant="gradient" asChild className="gap-2">
              <Link to="/dashboard">
                Launch App <ArrowRight className="h-4 w-4" />
              </Link>
            </Button>
            <Button size="lg" variant="outline" asChild>
              <Link to="/docs">Read Documentation</Link>
            </Button>
          </motion.div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-24 bg-secondary/5 border-t border-border/50">
        <div className="container px-4 md:px-6 max-w-screen-2xl">
          <div className="mb-12 text-center">
            <h2 className="text-3xl font-bold tracking-tight mb-4">Core Infrastructure for Builders</h2>
            <p className="text-muted-foreground max-w-[600px] mx-auto">
              Everything you need to build responsive, real-time decentralized applications.
            </p>
          </div>

          <motion.div 
            variants={containerVariants}
            initial="hidden"
            whileInView="visible"
            viewport={{ once: true, margin: "-100px" }}
            className="grid grid-cols-1 md:grid-cols-3 gap-8"
          >
            <motion.div variants={itemVariants} className="glass-card p-6 rounded-2xl">
              <div className="bg-indigo-500/10 w-12 h-12 rounded-lg flex items-center justify-center mb-6 border border-indigo-500/20">
                <Zap className="h-6 w-6 text-indigo-400" />
              </div>
              <h3 className="text-xl font-semibold mb-2">Instant Triggers</h3>
              <p className="text-muted-foreground text-sm leading-relaxed">
                Connect your smart contracts to Webhooks, Discord, or Telegram with sub-second latency the moment an event is emitted.
              </p>
            </motion.div>

            <motion.div variants={itemVariants} className="glass-card p-6 rounded-2xl">
              <div className="bg-emerald-500/10 w-12 h-12 rounded-lg flex items-center justify-center mb-6 border border-emerald-500/20">
                <Activity className="h-6 w-6 text-emerald-400" />
              </div>
              <h3 className="text-xl font-semibold mb-2">Reliable Polling</h3>
              <p className="text-muted-foreground text-sm leading-relaxed">
                Our distributed worker nodes ensure you never miss a block. Guaranteed delivery even during network congestion.
              </p>
            </motion.div>

            <motion.div variants={itemVariants} className="glass-card p-6 rounded-2xl">
              <div className="bg-purple-500/10 w-12 h-12 rounded-lg flex items-center justify-center mb-6 border border-purple-500/20">
                <Shield className="h-6 w-6 text-purple-400" />
              </div>
              <h3 className="text-xl font-semibold mb-2">Enterprise Security</h3>
              <p className="text-muted-foreground text-sm leading-relaxed">
                End-to-end webhook signature verification ensures the data hitting your servers is 100% authentic.
              </p>
            </motion.div>
          </motion.div>
        </div>
      </section>
    </div>
  )
}

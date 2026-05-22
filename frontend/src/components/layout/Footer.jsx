import React from 'react'

export function Footer() {
  return (
    <footer className="border-t border-border/40 py-6 md:py-0">
      <div className="container flex flex-col items-center justify-between gap-4 md:h-24 md:flex-row max-w-screen-2xl">
        <p className="text-center text-sm leading-loose text-muted-foreground md:text-left">
          Built by edohwares. The source code is available on GitHub.
        </p>
      </div>
    </footer>
  )
}

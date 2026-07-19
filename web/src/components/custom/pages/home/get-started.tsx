import { useLayoutEffect, useRef, useState, useSyncExternalStore } from "react"

import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Card, CardContent } from "@/components/ui/card"
import { cn } from "@/lib/utils"
import { CodeBlock } from "@/components/custom/code"
import { CopyToClipboard } from "@/components/custom/copy"

const commands = {
  cargo: ["cargo install bounds-cli"],
  homebrew: ["brew tap danfq/bounds https://github.com/danfq/bounds.git", "brew install danfq/bounds/bounds"],
} as const

type PackageManager = keyof typeof commands
type CardSize = { width: number; height: number }

const subscribeToHydration = () => () => {}

export const HomeGetStarted = () => {
  const [packageManager, setPackageManager] = useState<PackageManager>("cargo")
  const [cardSize, setCardSize] = useState<CardSize | null>(null)
  const hydrated = useSyncExternalStore(
    subscribeToHydration,
    () => true,
    () => false
  )
  const cardRefs = useRef<Record<PackageManager, HTMLDivElement | null>>({
    cargo: null,
    homebrew: null,
  })

  const changePackageManager = (value: string) => {
    if (!(value in commands)) return

    setPackageManager(value as PackageManager)
  }

  useLayoutEffect(() => {
    const updateCardSize = () => {
      const card = cardRefs.current[packageManager]
      if (!card) return

      const { width, height } = card.getBoundingClientRect()

      setCardSize((current) => {
        if (current?.width === width && current.height === height) return current

        return { width, height }
      })
    }

    updateCardSize()

    const observer = new ResizeObserver(updateCardSize)
    Object.values(cardRefs.current).forEach((card) => {
      if (card) observer.observe(card)
    })

    return () => observer.disconnect()
  }, [packageManager])

  return (
    <Tabs value={packageManager} onValueChange={changePackageManager} className="w-150 items-center gap-4">
      <TabsList>
        <TabsTrigger id="get-started-tab-cargo" value="cargo" aria-controls="get-started-panel">
          Cargo
        </TabsTrigger>
        <TabsTrigger id="get-started-tab-homebrew" value="homebrew" aria-controls="get-started-panel">
          Homebrew
        </TabsTrigger>
      </TabsList>

      <div
        id="get-started-panel"
        role="tabpanel"
        aria-labelledby={`get-started-tab-${packageManager}`}
        className="relative transition-[width,height] duration-300 ease-out motion-reduce:transition-none"
        style={cardSize ?? undefined}
      >
        {(Object.keys(commands) as PackageManager[]).map((manager) => {
          const active = manager === packageManager

          return (
            <div
              key={manager}
              ref={(card) => {
                cardRefs.current[manager] = card
              }}
              aria-hidden={!active}
              style={!hydrated && !active ? { opacity: 0, visibility: "hidden" } : undefined}
              className={cn(
                "w-max transition-[opacity,transform] duration-200 ease-out motion-reduce:transition-none",
                cardSize
                  ? "absolute top-0 left-1/2 -translate-x-1/2"
                  : active
                    ? "relative"
                    : "invisible absolute top-0 left-0",
                active ? "scale-100 opacity-100" : "pointer-events-none scale-[0.98] opacity-0"
              )}
            >
              <CommandCard key={manager} commands={[...commands[manager]]} />
            </div>
          )
        })}
      </div>
    </Tabs>
  )
}

/**
 * installation command card
 */
const CommandCard = ({ commands }: { commands: string[] }) => {
  return (
    <Card className="flex w-min flex-row">
      <CardContent className="text-sm text-muted-foreground">
        <div className="flex flex-col gap-2 whitespace-nowrap">
          {commands.map((command) => {
            return (
              <div className="flex flex-row items-center gap-2">
                <CodeBlock key={command} code={command} />
                <CopyToClipboard toCopy={command} />
              </div>
            )
          })}
        </div>
      </CardContent>
    </Card>
  )
}

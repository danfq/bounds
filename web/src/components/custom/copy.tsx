import { Button } from "@/components/ui/button"
import { Check, Copy } from "lucide-react"
import { useState } from "react"

export const CopyToClipboard = ({ toCopy }: { toCopy: string }) => {
  // handle copy
  const [copied, setCopied] = useState(false)
  const timeForCopiedIcon = 2000 // keep copied icon for 2 seconds
  const copyToClipboard = async (content: string) => {
    // copy to clipboard
    await navigator.clipboard.writeText(content.trim())

    // set copied
    setCopied(true)
    setTimeout(() => {
      setCopied(false)
    }, timeForCopiedIcon)
  }

  // ui
  return (
    <Button variant="ghost" size="icon" onClick={async () => await copyToClipboard(toCopy)}>
      {copied ? <Check /> : <Copy />}
    </Button>
  )
}

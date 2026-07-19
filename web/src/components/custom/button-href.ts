export type HrefHeaderRecord = Record<string, unknown>

const ALLOWED_HREF_HEADER_NAMES = new Set(["authorization", "accept-language"])
const ALLOWED_HREF_HEADER_PREFIXES = ["x-atoll-"]
const HREF_HEADERS_HISTORY_STATE = "__atollHrefHeadersNavigation"
let hasHrefHeadersPopstateHandler = false

function filenameFromHref(href: string): string {
  try {
    const { pathname } = new URL(href, window.location.href)
    const name = pathname.split("/").filter(Boolean).pop()
    return name ?? "download"
  } catch {
    return "download"
  }
}

function filenameFromContentDisposition(header: string | null): string | null {
  if (!header) return null
  const match = header.match(/filename\*?=(?:UTF-8''|")?([^";]+)/i)
  return match?.[1] ? decodeURIComponent(match[1].replace(/"/g, "")) : null
}

function isAllowedHrefHeaderName(headerName: string): boolean {
  const normalized = headerName.trim().toLowerCase()
  return (
    ALLOWED_HREF_HEADER_NAMES.has(normalized) ||
    ALLOWED_HREF_HEADER_PREFIXES.some((prefix) => normalized.startsWith(prefix))
  )
}

function isValidHrefHeaderName(headerName: string): boolean {
  return /^[!#$%&'*+\-.^_`|~0-9A-Za-z]+$/.test(headerName)
}

function isAllowedHrefHeaderValue(
  value: unknown
): value is string | number | boolean {
  return (
    typeof value === "string" ||
    typeof value === "number" ||
    typeof value === "boolean"
  )
}

function mergeHrefHeaders(hrefHeaders?: HrefHeaderRecord[]): Headers {
  const headers = new Headers()
  if (!hrefHeaders) return headers

  for (const record of hrefHeaders) {
    for (const [key, value] of Object.entries(record)) {
      const trimmedKey = key.trim()
      if (!trimmedKey || !isValidHrefHeaderName(trimmedKey)) continue
      if (!isAllowedHrefHeaderName(trimmedKey)) continue
      if (value == null || !isAllowedHrefHeaderValue(value)) continue
      headers.set(trimmedKey, String(value))
    }
  }

  return headers
}

export function hasHrefHeaders(
  hrefHeaders?: HrefHeaderRecord[]
): hrefHeaders is HrefHeaderRecord[] {
  return hrefHeaders != null && hrefHeaders.length > 0
}

function ensureHrefHeadersPopstateHandler() {
  if (hasHrefHeadersPopstateHandler) return
  hasHrefHeadersPopstateHandler = true

  window.addEventListener("popstate", () => {
    window.location.reload()
  })
}

function triggerBlobDownload(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement("a")
  anchor.href = url
  anchor.download = filename
  anchor.rel = "noopener noreferrer"
  anchor.style.display = "none"
  document.body.appendChild(anchor)
  anchor.click()
  document.body.removeChild(anchor)
  URL.revokeObjectURL(url)
}

function downloadFromHref(href: string) {
  const anchor = document.createElement("a")
  anchor.href = href
  anchor.download = filenameFromHref(href)
  anchor.rel = "noopener noreferrer"
  anchor.style.display = "none"
  document.body.appendChild(anchor)
  anchor.click()
  document.body.removeChild(anchor)
}

async function downloadFromHrefWithHeaders(
  href: string,
  hrefHeaders: HrefHeaderRecord[]
) {
  const res = await fetch(href, {
    headers: mergeHrefHeaders(hrefHeaders),
    credentials: "include",
  })

  if (!res.ok) return

  const blob = await res.blob()
  const filename =
    filenameFromContentDisposition(res.headers.get("content-disposition")) ??
    filenameFromHref(href)
  triggerBlobDownload(blob, filename)
}

async function navigateToHrefWithHeaders(
  href: string,
  hrefHeaders: HrefHeaderRecord[]
) {
  const res = await fetch(href, {
    headers: mergeHrefHeaders(hrefHeaders),
    credentials: "include",
    redirect: "follow",
  })

  if (!res.ok) {
    window.location.assign(href)
    return
  }

  if (res.redirected) {
    window.location.assign(res.url)
    return
  }

  const contentType = res.headers.get("content-type") ?? ""
  if (!contentType.includes("text/html")) {
    window.location.assign(href)
    return
  }

  let resolvedUrl: URL
  try {
    resolvedUrl = new URL(res.url || href, window.location.href)
  } catch {
    window.location.assign(href)
    return
  }

  if (resolvedUrl.origin !== window.location.origin) {
    window.location.assign(href)
    return
  }

  ensureHrefHeadersPopstateHandler()
  window.history.replaceState(
    { ...(window.history.state ?? {}), [HREF_HEADERS_HISTORY_STATE]: true },
    ""
  )
  window.history.pushState(
    { [HREF_HEADERS_HISTORY_STATE]: true },
    "",
    `${resolvedUrl.pathname}${resolvedUrl.search}${resolvedUrl.hash}`
  )
  const html = await res.text()
  document.open()
  document.write(html)
  document.close()
}

async function openHrefWithHeadersInNewTab(
  href: string,
  hrefHeaders: HrefHeaderRecord[]
) {
  const res = await fetch(href, {
    headers: mergeHrefHeaders(hrefHeaders),
    credentials: "include",
    redirect: "follow",
  })

  if (!res.ok) {
    window.open(href, "_blank", "noopener,noreferrer")
    return
  }

  if (res.redirected) {
    window.open(res.url, "_blank", "noopener,noreferrer")
    return
  }

  const blob = await res.blob()
  const url = URL.createObjectURL(blob)
  window.open(url, "_blank", "noopener,noreferrer")
  setTimeout(() => URL.revokeObjectURL(url), 0)
}

type FollowButtonHrefOptions = {
  href: string
  hrefHeaders?: HrefHeaderRecord[]
  autoDownload?: boolean
  hrefTarget?: "_blank" | "_self"
  openInNewTab: boolean
}

export async function followButtonHref({
  href,
  hrefHeaders,
  autoDownload = false,
  hrefTarget = "_self",
  openInNewTab,
}: FollowButtonHrefOptions) {
  if (hasHrefHeaders(hrefHeaders)) {
    if (autoDownload) {
      await downloadFromHrefWithHeaders(href, hrefHeaders)
      return
    }
    if (openInNewTab) {
      await openHrefWithHeadersInNewTab(href, hrefHeaders)
      return
    }
    await navigateToHrefWithHeaders(href, hrefHeaders)
    return
  }

  if (autoDownload) {
    if (openInNewTab) {
      window.open(href, hrefTarget, "noopener,noreferrer")
      return
    }
    downloadFromHref(href)
    return
  }
  if (openInNewTab) {
    window.open(href, hrefTarget, "noopener,noreferrer")
    return
  }
  window.location.assign(href)
}

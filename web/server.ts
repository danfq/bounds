const dist = new URL("./dist/", import.meta.url)

const server = Bun.serve({
  hostname: "0.0.0.0",
  port: Number(Bun.env.PORT ?? 3000),

  async fetch(request) {
    const url = new URL(request.url)
    const pathname = decodeURIComponent(url.pathname)
    const relativePath = pathname === "/" ? "index.html" : pathname.replace(/^\/+/, "")
    const fileUrl = new URL(relativePath, dist)

    if (!fileUrl.href.startsWith(dist.href)) {
      return new Response("Not found", { status: 404 })
    }

    const file = Bun.file(fileUrl)
    if (await file.exists()) {
      return new Response(file)
    }

    if (!relativePath.includes(".")) {
      const indexUrl = new URL(`${relativePath.replace(/\/?$/, "/")}index.html`, dist)
      const index = Bun.file(indexUrl)

      if (indexUrl.href.startsWith(dist.href) && (await index.exists())) {
        return new Response(index)
      }
    }

    return new Response("Not found", { status: 404 })
  },
})

console.log(`Serving ${dist.pathname} at ${server.url}`)

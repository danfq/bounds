/**
 * renders a code block
 *
 * @param code code to render
 */
export const CodeBlock = ({ code }: { code: string }) => {
  return <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-sm">{code}</code>
}

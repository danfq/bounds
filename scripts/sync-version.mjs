import { readFile, writeFile } from "node:fs/promises"
import { fileURLToPath } from "node:url"

const root = new URL("../", import.meta.url)
const cargoPath = new URL("Cargo.toml", root)
const packagePath = new URL("web/package.json", root)
const checkOnly = process.argv.includes("--check")

const cargo = await readFile(cargoPath, "utf8")
const packageSectionStart = cargo.indexOf("[package]")

if (packageSectionStart === -1) {
  throw new Error("Cargo.toml does not contain a [package] section")
}

const afterPackage = cargo.slice(packageSectionStart + "[package]".length)
const nextSection = afterPackage.search(/\n\[/)
const packageSection =
  nextSection === -1 ? afterPackage : afterPackage.slice(0, nextSection)
const version = packageSection.match(/^version\s*=\s*"([^"]+)"\s*$/m)?.[1]

if (!version || !/^\d+\.\d+\.\d+$/.test(version)) {
  throw new Error(
    "Cargo.toml package version must use stable major.minor.patch SemVer"
  )
}

const packageJson = JSON.parse(await readFile(packagePath, "utf8"))
const previousVersion = packageJson.version

if (previousVersion === version) {
  console.log(`Versions already match at ${version}`)
  process.exit(0)
}

if (checkOnly) {
  console.error(
    `Version mismatch: Cargo.toml is ${version}, web/package.json is ${previousVersion}`
  )
  process.exit(1)
}

packageJson.version = version
await writeFile(packagePath, `${JSON.stringify(packageJson, null, 2)}\n`)

console.log(
  `Updated ${fileURLToPath(packagePath)} from ${previousVersion} to ${version}`
)

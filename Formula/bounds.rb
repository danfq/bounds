class Bounds < Formula
  desc "Add a .gitignore and LICENSE to a repository"
  homepage "https://github.com/danfq/bounds"
  url "https://github.com/danfq/bounds.git",
      tag:      "v0.4.0",
      revision: "377f29cff9e6478b151b4a5bf72c93be1ec2e882"
  license "AGPL-3.0-only"
  head "https://github.com/danfq/bounds.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    missing = testpath/"missing"
    output = shell_output("#{bin}/bounds #{missing} 2>&1", 1)

    assert_match "is not a directory", output
  end
end

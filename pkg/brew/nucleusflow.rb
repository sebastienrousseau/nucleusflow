# SPDX-License-Identifier: Apache-2.0 OR MIT
class Nucleusflow < Formula
  desc "High-performance, memory-safe static site generator written in Rust"
  homepage "https://nucleusflow.com"
  url "https://github.com/sebastienrousseau/nucleusflow/archive/refs/tags/v0.0.7.tar.gz"
  license any_of: ["Apache-2.0", "MIT"]
  head "https://github.com/sebastienrousseau/nucleusflow.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
    man1.install "share/man/man1/nucleusflow.1"
    bash_completion.install "completions/bash/nucleusflow.bash" => "nucleusflow"
    zsh_completion.install "completions/zsh/_nucleusflow"
    fish_completion.install "completions/fish/nucleusflow.fish"
  end

  test do
    system "#{bin}/nucleusflow", "--version"
    system "#{bin}/nucleusflow", "new", "testsite"
    assert_predicate testpath/"testsite/nucleusflow.toml", :exist?
  end
end

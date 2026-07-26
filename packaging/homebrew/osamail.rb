class Osamail < Formula
  desc "Tiny, scriptable CLI for Apple Mail"
  homepage "https://github.com/tinylion1024/osamail"
  url "https://github.com/tinylion1024/osamail/releases/download/v0.1.0/osamail-v0.1.0-universal-apple-darwin.tar.gz"
  sha256 "ece067e21bf9ca48b68790844898c2f471cbc034b27eafd616f0591466826f74"
  version "0.1.0"
  license "MIT"

  depends_on :macos

  def install
    bin.install "osamail"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/osamail --version")
    assert_match "Usage:", shell_output("#{bin}/osamail --help")
  end
end

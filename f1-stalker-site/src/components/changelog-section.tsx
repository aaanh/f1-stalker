import { useEffect, useState } from "react"
import ReactMarkdown from "react-markdown"
import type { Components } from "react-markdown"
import { changelogBrowseUrl, fetchChangelog } from "@/lib/changelog"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "./ui/dialog"
import { Button } from "./ui/button"

const markdownComponents: Components = {
  a: ({ href, children }) => (
    <a href={href} target="_blank" rel="noopener noreferrer">
      {children}
    </a>
  ),
}

export function ChangelogSection() {
  const [content, setContent] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false

    fetchChangelog()
      .then((markdown) => {
        if (!cancelled) {
          setContent(markdown)
        }
      })
      .catch((cause: unknown) => {
        if (!cancelled) {
          const message =
            cause instanceof Error ? cause.message : "Failed to load changelog"
          setError(message)
        }
      })

    return () => {
      cancelled = true
    }
  }, [])

  return (
    <>
      <h2>Changelog</h2>
      <Dialog>
        <DialogTrigger asChild>
          <Button>View changelog</Button>
        </DialogTrigger>
        <DialogContent className="max-w-[min(80vw,calc(100%-2rem))] sm:max-w-[min(80vw,calc(100%-2rem))]">
          <DialogHeader>
            <DialogTitle>F1 Stalker Changelog</DialogTitle>
            <DialogDescription>
              Loaded from{" "}
              <a
                href={changelogBrowseUrl()}
                target="_blank"
                rel="noopener noreferrer"
              >
                GitLab
              </a>{" "}
              on page load (always in sync with the repository).
            </DialogDescription>
          </DialogHeader>
          <div className="prose max-w-none max-h-[80svh] w-full overflow-auto dark:prose-invert">
            {error ? (
              <p>
                Could not load changelog ({error}). View it on{" "}
                <a
                  href={changelogBrowseUrl()}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  GitLab
                </a>
                .
              </p>
            ) : null}
            {!content && !error ? <p>Loading changelog…</p> : null}
            {content ? (
              <div className="changelog-markdown">
                <ReactMarkdown components={markdownComponents}>
                  {content}
                </ReactMarkdown>
              </div>
            ) : null}
          </div>
        </DialogContent>
      </Dialog>
    </>
  )
}

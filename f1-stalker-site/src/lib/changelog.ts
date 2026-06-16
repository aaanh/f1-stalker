const GITLAB_PROJECT_PATH = "aaanh/f1-stalker"
const CHANGELOG_FILE = "CHANGELOG.md"
const DEFAULT_REF = "master"

export function changelogRawUrl(ref = DEFAULT_REF): string {
  const project = encodeURIComponent(GITLAB_PROJECT_PATH)
  const file = encodeURIComponent(CHANGELOG_FILE)
  return `https://gitlab.com/api/v4/projects/${project}/repository/files/${file}/raw?ref=${encodeURIComponent(ref)}`
}

export function changelogBrowseUrl(ref = DEFAULT_REF): string {
  return `https://gitlab.com/${GITLAB_PROJECT_PATH}/-/blob/${ref}/${CHANGELOG_FILE}`
}

function decodeHtmlEntities(markdown: string): string {
  return markdown
    .replaceAll("&ndash;", "–")
    .replaceAll("&rarr;", "→")
    .replaceAll("&ldquo;", "“")
    .replaceAll("&rdquo;", "”")
    .replaceAll("&amp;", "&")
}

/** Drop Keep a Changelog boilerplate; start at the first version section. */
export function prepareChangelogMarkdown(markdown: string): string {
  const decoded = decodeHtmlEntities(markdown)
  const start = decoded.search(/^## \[/m)
  return start >= 0 ? decoded.slice(start).trimEnd() : decoded.trim()
}

export async function fetchChangelog(ref = DEFAULT_REF): Promise<string> {
  const response = await fetch(changelogRawUrl(ref))

  if (!response.ok) {
    throw new Error(`Failed to load changelog (${response.status})`)
  }

  const markdown = await response.text()
  return prepareChangelogMarkdown(markdown)
}

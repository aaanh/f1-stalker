import { APP_VERSION, releaseDownloads } from "@/lib/releases"
import { Logo } from "@/components/logo"
import { ButtonGroup } from "./components/ui/button-group"
import { Button, buttonVariants } from "./components/ui/button"

export function App() {
  return (
    <>
      <head>
        <title>F1 Stalker</title>
        <link rel="icon" type="image/png" sizes="32x32" href="/favicon.png" />
        <link
          rel="apple-touch-icon"
          sizes="180x180"
          href="/apple-touch-icon.png"
        />
        <meta
          name="description"
          content="Track Formula One race results, standings, and upcoming races. Built for fans who live and breathe the drama."
        />
        <meta name="application-name" content="F1 Stalker" />
        <meta name="theme-color" content="#b91c1c" />
        <meta property="og:title" content="F1 Stalker" />
        <meta
          property="og:description"
          content="Track Formula One race results, standings, and upcoming races. Built for fans who live and breathe the drama."
        />
        <meta property="og:type" content="website" />
        <meta property="og:site_name" content="F1 Stalker" />
        <meta name="twitter:card" content="summary" />
        <meta name="twitter:title" content="F1 Stalker" />
        <meta
          name="twitter:description"
          content="Track Formula One race results, standings, and upcoming races. Built for fans who live and breathe the drama."
        />
      </head>
      <div className="flex min-h-svh w-full flex-col bg-neutral-800">
        <header className="fixed flex w-full items-center justify-between gap-2 bg-background p-4">
          <div className="flex items-center gap-2">
            <Logo
              className="rounded-full border border-red-500 p-1"
              size={48}
            />
            <h1 className="text-4xl">F1 Stalker</h1>
          </div>
          <ButtonGroup className="flex flex-wrap">
            <a
              className={buttonVariants({ variant: "default" })}
              href="https://gitlab.com/aaanh/f1-stalker"
            >
              GitLab (F1 Stalker)
            </a>
            <a
              className={buttonVariants({ variant: "default" })}
              href="https://gitlab.com/aaanh/f1-client"
            >
              GitLab (OpenF1 Client)
            </a>
          </ButtonGroup>
        </header>
        <div className="container mx-auto py-24 pb-12">
          <main className="mx-auto prose rounded-lg bg-neutral-900 p-4 lg:prose-xl dark:prose-invert">
            <h2>What?</h2>
            <p>
              {`You can't get enough of Formula One (TM)? You live and breathe the
              drama, the bottling, and the comedy? Then you're probably just
              like me. That's why I built this app to track the results and
              upcoming races. And if you're sadistically masochistic, you
              would have built it in Rust, like I did, too!`}
            </p>
            <h2>Downloads</h2>
            <p>
              Latest release: <strong>v{APP_VERSION}</strong>
            </p>
            <ButtonGroup>
              {releaseDownloads.map((download) => (
                <Button key={download.href} asChild>
                  <a href={download.href} download={download.fileName}>
                    {download.label}
                  </a>
                </Button>
              ))}
            </ButtonGroup>
            <h2>Screenshots</h2>
            <img
              className="rounded-lg border border-red-500 p-1 shadow-lg"
              src="/f1-stalker-screenshot-0.png"
              width={800}
              height={600}
              alt="F1 Stalker dashboard screenshot"
            />
            <img
              className="rounded-lg border border-red-500 p-1 shadow-lg"
              src="/f1-stalker-screenshot-1.png"
              width={800}
              height={600}
              alt="F1 Stalker championship screenshot"
            />
            <h2>Troubleshoot</h2>
            <p>
              <b>macOS Gatekeeper blocks the app from running:</b> Right-click
              on the app, click <kbd>Open</kbd> again. If still not works, go to
              Settings <kbd>&rarr;</kbd> Security & Privacy <kbd>&rarr;</kbd>{" "}
              Scroll down until you see the allow this app to run button.
            </p>
            <h2>Legal and Disclaimer</h2>
            <p>
              This website is unofficial and is not associated in any way with
              the Formula 1 companies. F1, FORMULA ONE, FORMULA 1, FIA FORMULA
              ONE WORLD CHAMPIONSHIP, GRAND PRIX and related marks are trade
              marks of Formula One Licensing B.V.
            </p>
            <p>
              This site and the owner/creator of this site are not affiliated
              with the drivers nor the constructors and their sponsors.
            </p>
            <p>
              Please contact the site owner via e-mail at{" "}
              <pre>iam (at) hoanganh (dot) dev</pre> for copyright and trademark
              infringement concerns.
            </p>
            <p>
              The application F1 Stalker utilizes OpenF1 API to fetch race data.
              It currently only supports data that is 24-hour old. In the
              future, there will be an option to supply your paid API key to
              fetch live or near-live data.
            </p>
          </main>
        </div>
      </div>
      <footer className="flex h-32 flex-col items-center justify-center gap-2">
        <p className="text-center">Copyright &copy; 2026 Anh H. Nguyen</p>
        <ButtonGroup>
          <a
            className={buttonVariants({ variant: "outline" })}
            href="https://linkedin.com/in/aaanh"
          >
            Linkedin
          </a>
          <a
            className={buttonVariants({ variant: "outline" })}
            href="https://aaanh.com"
          >
            Homepage
          </a>
          <a
            className={buttonVariants({ variant: "outline" })}
            href="mailto:iam@hoanganh.dev"
          >
            Email
          </a>
        </ButtonGroup>
      </footer>
    </>
  )
}

export default App

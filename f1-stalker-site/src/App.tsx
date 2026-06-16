import { APP_VERSION, releaseDownloads } from "@/lib/releases"
import { ChangelogSection } from "@/components/changelog-section"
import { Logo } from "@/components/logo"
import { ButtonGroup } from "./components/ui/button-group"
import { buttonVariants } from "./components/ui/button"
import React from "react"
import { cn } from "./lib/utils"

export function App() {
  return (
    <>
      <div className="flex min-h-svh w-full flex-col bg-neutral-800">
        <header className="fixed flex w-full items-center justify-between gap-2 bg-background p-4">
          <div className="flex items-center gap-2">
            <Logo
              className="rounded-full border border-red-500 p-1"
              size={48}
            />
            <h1 className="hidden text-4xl sm:block">F1 Stalker</h1>
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
              href="https://gitlab.com/aaanh/openf1-client"
            >
              GitLab (OpenF1 Client)
            </a>
          </ButtonGroup>
        </header>
        <div className="container mx-auto px-2 py-24 pb-12">
          <main className="mx-auto prose rounded-lg bg-neutral-800 p-4 sm:bg-neutral-900 lg:prose-xl dark:prose-invert">
            <h2>What is F1 Stalker?</h2>
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
            <ButtonGroup className="flex-wrap gap-2">
              {releaseDownloads.map((download) => (
                <a
                  className={cn(
                    buttonVariants({ variant: "default" }),
                    "no-underline hover:no-underline"
                  )}
                  key={download.href}
                  href={download.href}
                  download={download.fileName}
                >
                  {download.label}
                </a>
              ))}
            </ButtonGroup>
            <ChangelogSection />
            <h2>Demo video</h2>
            <iframe
              src="https://www.linkedin.com/embed/feed/update/urn:li:ugcPost:7471070502682411008?collapsed=1"
              height="720"
              width="100%"
              allowFullScreen={true}
              className="w-full"
              title="Embedded post"
            ></iframe>
            <h2>Features</h2>
            <ul>
              <li>
                <strong>Season at a glance:</strong> previous, current, and
                upcoming races with circuit details, session times, and a
                countdown to the next on-track action.
              </li>
              <li>
                <strong>Pinned drivers:</strong> follow as many drivers as you
                like and track their championship progress on interactive
                charts. Constructor standings follow the teams behind your
                pins.
              </li>
              <li>
                <strong>Standings table:</strong> full drivers and constructors
                grids with a toggle for championship points or the latest race
                result.
              </li>
              <li>
                <strong>Rival mode:</strong> pick two drivers and compare
                head-to-head stats, gaps, and chart focus without losing sight
                of the wider season.
              </li>
              <li>
                <strong>Race weekend detail:</strong> qualifying and sprint
                starting grids for pinned drivers, plus weather forecasts and
                track conditions where available.
              </li>
              <li>
                <strong>Themes:</strong> dark, light, and constructor-inspired
                color presets that apply across the whole dashboard.
              </li>
              <li>
                <strong>Stay in the loop:</strong> desktop notifications for
                pinned-driver standings changes, optional session reminders, and
                a system tray so the app can run in the background.
              </li>
              <li>
                <strong>Native and cross-platform:</strong> built in Rust with
                Iced for macOS, Windows, and Linux. Data is cached locally in
                SQLite so the dashboard stays usable offline.
              </li>
            </ul>
            <p>
              Race data comes from the free OpenF1 historical API (about a
              24-hour delay). Live timing is planned for a future release.
            </p>
            <h2>Screenshots</h2>
            {[...Array(6).keys()].map((num) => (
              <React.Fragment key={`screenshot-${num}`}>
                <p className="text-2xl">
                  <strong>P{num + 1}</strong>
                </p>
                <img
                  className="rounded-lg border border-red-500 p-1 shadow-lg"
                  src={`/f1-stalker-screenshot-${num}.png`}
                  width={800}
                  height={600}
                  alt="F1 Stalker dashboard screenshot"
                />
              </React.Fragment>
            ))}
            <h2>Troubleshoot</h2>
            <h3>macOS says the app cannot be opened</h3>
            <p>
              F1 Stalker is not signed with an Apple Developer certificate, so
              macOS Gatekeeper may block the downloaded app the first time you
              open it. This is expected for indie releases and does not mean the
              app is broken.
            </p>
            <p>
              <strong>Try this first:</strong> In Finder, right-click (or
              Control-click) <kbd>F1 Stalker.app</kbd>, choose{" "}
              <kbd>Open</kbd>, then confirm <kbd>Open</kbd> in the dialog. You
              only need to do this once.
            </p>
            <p>
              <strong>If that does not work:</strong> Open{" "}
              <kbd>System Settings</kbd> <kbd>&rarr;</kbd>{" "}
              <kbd>Privacy &amp; Security</kbd>, scroll to the{" "}
              <kbd>Security</kbd> section, and click{" "}
              <kbd>Open Anyway</kbd> next to the F1 Stalker message. Enter your
              password if macOS asks for it, then open the app again.
            </p>
            <p>
              Still stuck? Make sure you unzipped the download and are opening
              the <kbd>.app</kbd> bundle, not the <kbd>.dmg</kbd> or{" "}
              <kbd>.zip</kbd> file itself.
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
            <p>Please contact the site owner via e-mail at </p>
            <pre>iam (at) hoanganh (dot) dev</pre> for copyright and trademark
            infringement concerns.
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
        <p className="flex flex-wrap items-center gap-1 text-center">
          <a href="https://f1stalker.aaanh.com">F1 Stalker</a> © 2026 by{" "}
          <a href="https://aaanh.com">Anh Nguyen</a> is licensed under{" "}
          <a href="https://creativecommons.org/licenses/by-sa/4.0/">
            CC BY-SA 4.0
          </a>
          <img
            src="https://mirrors.creativecommons.org/presskit/icons/cc.svg"
            alt=""
            className="ml-[0.2em] max-h-[1em] max-w-[1em]"
          />
          <img
            src="https://mirrors.creativecommons.org/presskit/icons/by.svg"
            alt=""
            className="ml-[0.2em] max-h-[1em] max-w-[1em]"
          />
          <img
            src="https://mirrors.creativecommons.org/presskit/icons/sa.svg"
            alt=""
            className="ml-[0.2em] max-h-[1em] max-w-[1em]"
          />
        </p>
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

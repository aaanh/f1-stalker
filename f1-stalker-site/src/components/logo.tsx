import * as React from "react"

import appLogo from "@/assets/app-logo.png"
import { cn } from "@/lib/utils"

type LogoProps = Omit<React.ComponentProps<"img">, "src" | "alt"> & {
  alt?: string
  size?: number
}

export function Logo({
  alt = "F1 Stalker",
  className,
  size = 32,
  width,
  height,
  ...props
}: LogoProps) {
  return (
    <img
      src={appLogo}
      alt={alt}
      width={width ?? size}
      height={height ?? size}
      className={cn("shrink-0", className)}
      {...props}
    />
  )
}

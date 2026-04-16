import { APP_LOGO_ALT } from "@/lib/appIdentity";

type AppLogoProps = {
  className?: string;
};

export default function AppLogo({ className = "h-10 w-10" }: AppLogoProps) {
  return (
    <img
      src="/app-logo.png"
      alt={APP_LOGO_ALT}
      className={`${className} object-contain`}
    />
  );
}

type AppLogoProps = {
  className?: string;
};

export default function AppLogo({ className = "h-10 w-10" }: AppLogoProps) {
  return (
    <img
      src="/app-logo.png"
      alt="App logo"
      className={`${className} object-contain`}
    />
  );
}

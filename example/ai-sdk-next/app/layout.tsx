import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Local AI Gateway AI SDK Test",
  description: "Next.js example for testing Local AI Gateway with Vercel AI SDK.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}

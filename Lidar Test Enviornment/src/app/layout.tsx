import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({
  subsets: ["latin"],
  variable: "--font-inter",
});

export const metadata: Metadata = {
  title: "Sennep SLAM Suite | LiDAR Mapping",
  description: "2D LiDAR Mapping & SLAM Suite with RANSAC, Occupancy Grid, and ICP Scan Matching",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="h-full dark">
      <body className={`${inter.variable} h-full m-0 p-0 overflow-hidden`}>
        {children}
      </body>
    </html>
  );
}

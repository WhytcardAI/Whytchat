import { clsx } from "clsx";
import { twMerge } from "tailwind-merge";

/**
 * Utility to merge Tailwind classes safely
 * @param {...string} inputs
 * @returns {string}
 */
export function cn(...inputs) {
  return twMerge(clsx(inputs));
}

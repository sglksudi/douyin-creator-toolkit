import * as React from "react";
import { cn } from "@/lib/utils";

const Input = React.forwardRef<HTMLInputElement, React.ComponentProps<"input">>(
  ({ className, type, ...props }, ref) => {
    return (
      <input
        type={type}
        className={cn(
          "flex h-10 w-full rounded-lg border border-zinc-200 bg-white px-4 py-2 text-sm transition-all duration-200 file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-zinc-400 focus:border-[#1976D2] focus:ring-2 focus:ring-[#1976D2]/20 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50 dark:border-zinc-700 dark:bg-zinc-900 dark:placeholder:text-zinc-500 dark:focus:border-[#42A5F5] dark:focus:ring-[#42A5F5]/20",
          className
        )}
        ref={ref}
        {...props}
      />
    );
  }
);
Input.displayName = "Input";

// Textarea component
const Textarea = React.forwardRef<HTMLTextAreaElement, React.ComponentProps<"textarea">>(
  ({ className, ...props }, ref) => {
    return (
      <textarea
        className={cn(
          "flex min-h-[100px] w-full rounded-lg border border-zinc-200 bg-white px-4 py-3 text-sm transition-all duration-200 placeholder:text-zinc-400 focus:border-[#1976D2] focus:ring-2 focus:ring-[#1976D2]/20 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50 dark:border-zinc-700 dark:bg-zinc-900 dark:placeholder:text-zinc-500 dark:focus:border-[#42A5F5] dark:focus:ring-[#42A5F5]/20 resize-none",
          className
        )}
        ref={ref}
        {...props}
      />
    );
  }
);
Textarea.displayName = "Textarea";

export { Input, Textarea };

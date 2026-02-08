import { motion } from "motion/react";
import { pageVariants } from "@/lib/animations";
import { cn } from "@/lib/utils";

interface MainContentProps {
  title: string;
  description?: string;
  /** Optional slot on the right side of the header (e.g. refresh icon). */
  actions?: React.ReactNode;
  /** Optional class for the title (e.g. text-base for a smaller title). */
  titleClassName?: string;
  children?: React.ReactNode;
}

export function MainContent({
  title,
  description,
  actions,
  titleClassName,
  children,
}: MainContentProps) {
  return (
    <main className="ml-[200px] min-h-screen flex flex-col flex-1">
      <motion.div
        initial="initial"
        animate="animate"
        exit="exit"
        variants={pageVariants}
        transition={{ duration: 0.2 }}
        className="flex-1 p-5"
      >
        {/* Page Header - Fixed Height */}
        <div className="h-12 flex flex-col justify-center">
          <div className="flex items-start justify-between gap-2">
            <div className="min-w-0 flex-1">
              <h1 className={cn("text-lg font-semibold tracking-tight", titleClassName)}>{title}</h1>
              {description && (
                <p className="text-sm text-muted-foreground truncate">{description}</p>
              )}
            </div>
            {actions ? <div className="shrink-0">{actions}</div> : null}
          </div>
        </div>

        {/* Separator */}
        <div className="border-b border-border my-4" />

        {/* Page Content */}
        {children}
      </motion.div>
    </main>
  );
}

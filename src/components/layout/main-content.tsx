import { motion } from "motion/react";
import { pageVariants } from "@/lib/animations";

interface MainContentProps {
  title: string;
  description?: string;
  children?: React.ReactNode;
}

export function MainContent({ 
  title, 
  description, 
  children 
}: MainContentProps) {
  return (
    <main className="ml-[180px] min-h-screen flex flex-col flex-1">
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
          <h1 className="text-lg font-semibold tracking-tight">{title}</h1>
          {description && (
            <p className="text-xs text-muted-foreground truncate">{description}</p>
          )}
        </div>

        {/* Separator */}
        <div className="border-b border-border my-4" />

        {/* Page Content */}
        {children}
      </motion.div>
    </main>
  );
}

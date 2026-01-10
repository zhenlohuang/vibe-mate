import type { Variants } from "motion/react";

// Page transition animation
export const pageVariants: Variants = {
  initial: { opacity: 0, y: 20 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -20 },
};

// Card list animation
export const containerVariants: Variants = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: {
      staggerChildren: 0.1,
    },
  },
};

export const itemVariants: Variants = {
  hidden: { opacity: 0, y: 20 },
  show: { opacity: 1, y: 0 },
};

// Drag rule animation
export const dragVariants: Variants = {
  idle: { scale: 1, boxShadow: "0 0 0 rgba(0,0,0,0)" },
  dragging: {
    scale: 1.02,
    boxShadow: "0 10px 30px rgba(168, 85, 247, 0.3)",
    transition: { duration: 0.2 },
  },
};

// Sidebar menu item animation
export const menuItemVariants: Variants = {
  inactive: {
    backgroundColor: "transparent",
    x: 0,
  },
  active: {
    backgroundColor: "rgba(168, 85, 247, 0.1)",
    x: 4,
    transition: { duration: 0.2 },
  },
};

// Hover lift animation
export const hoverLiftVariants: Variants = {
  initial: { y: 0 },
  hover: {
    y: -4,
    transition: { duration: 0.2 },
  },
};


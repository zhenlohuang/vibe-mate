import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Toaster } from "@/components/ui/toaster";
import { Sidebar } from "@/components/layout/sidebar";
import { useProxyStatus } from "@/hooks/use-tauri";
import {
  DashboardPage,
  ProvidersPage,
  RouterPage,
  AgentsPage,
  AgentConfigPage,
  SettingsPage,
} from "@/pages";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5, // 5 minutes
      retry: 1,
    },
  },
});

// Component that initializes global polling hooks
function AppInitializer() {
  // Start polling proxy status on app load
  useProxyStatus();
  return null;
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <TooltipProvider>
        <BrowserRouter>
          <AppInitializer />
          <div className="flex min-h-screen bg-background">
            <Sidebar />
            <Routes>
              <Route path="/" element={<DashboardPage />} />
              <Route path="/providers" element={<ProvidersPage />} />
              <Route path="/router" element={<RouterPage />} />
              <Route path="/agents" element={<AgentsPage />} />
              <Route path="/agents/:agentType/config" element={<AgentConfigPage />} />
              <Route path="/settings" element={<SettingsPage />} />
              {/* Legacy route redirects */}
              <Route path="/general" element={<Navigate to="/settings" replace />} />
              <Route path="/network" element={<Navigate to="/settings" replace />} />
            </Routes>
          </div>
          <Toaster />
        </BrowserRouter>
      </TooltipProvider>
    </QueryClientProvider>
  );
}

export default App;

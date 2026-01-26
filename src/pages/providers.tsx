import { useState, useMemo } from "react";
import { motion, AnimatePresence } from "motion/react";
import { Plus, Loader2 } from "lucide-react";
import { MainContent } from "@/components/layout/main-content";
import {
  ProviderCard,
  ProviderForm,
  AgentProviderForm,
} from "@/components/providers";
import { useProviders } from "@/hooks/use-providers";
import { useToast } from "@/hooks/use-toast";
import type {
  Provider,
  CreateProviderInput,
  UpdateProviderInput,
  ProviderCategory,
  AgentProviderType,
} from "@/types";
import { containerVariants, itemVariants } from "@/lib/animations";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";

export function ProvidersPage() {
  const {
    providers,
    isLoading,
    createProvider,
    updateProvider,
    deleteProvider,
    testConnection,
  } = useProviders();
  const { toast } = useToast();

  const [category, setCategory] = useState<ProviderCategory>("Model");
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [isAgentFormOpen, setIsAgentFormOpen] = useState(false);
  const [editingProvider, setEditingProvider] = useState<
    Provider | undefined
  >();
  const [editingAgentProvider, setEditingAgentProvider] = useState<
    Provider | undefined
  >();

  const existingAgentTypes = useMemo(
    () =>
      providers
        .filter((p) => p.category === "Agent")
        .map((p) => p.type as AgentProviderType),
    [providers],
  );

  const filteredProviders = useMemo(
    () => providers.filter((p) => p.category === category),
    [providers, category],
  );

  const handleCreate = async (data: CreateProviderInput) => {
    try {
      await createProvider(data);
      toast({
        title: "Provider Created",
        description: `${data.name} has been added successfully.`,
      });
    } catch (error) {
      toast({
        title: "Error",
        description: String(error),
        variant: "destructive",
      });
      throw error;
    }
  };

  const handleEdit = (provider: Provider) => {
    setEditingProvider(provider);
    setIsFormOpen(true);
  };

  const handleUpdate = async (data: CreateProviderInput) => {
    if (!editingProvider) return;
    try {
      const updateData: UpdateProviderInput = {
        name: data.name,
        apiBaseUrl: data.apiBaseUrl,
      };
      if (data.apiKey) {
        updateData.apiKey = data.apiKey;
      }
      await updateProvider(editingProvider.id, updateData);
      toast({
        title: "Provider Updated",
        description: `${data.name} has been updated successfully.`,
      });
    } catch (error) {
      toast({
        title: "Error",
        description: String(error),
        variant: "destructive",
      });
      throw error;
    }
  };

  const handleDelete = async (id: string) => {
    const provider = providers.find((p) => p.id === id);
    try {
      await deleteProvider(id);
      toast({
        title: "Provider Deleted",
        description: `${provider?.name || "Provider"} has been removed.`,
      });
    } catch (error) {
      toast({
        title: "Error",
        description: String(error),
        variant: "destructive",
      });
      throw error;
    }
  };

  const handleTestConnection = async (id: string) => {
    const provider = providers.find((p) => p.id === id);
    try {
      const result = await testConnection(id);
      if (result.isConnected) {
        toast({
          title: "Connection Successful",
          description: `${provider?.name} is reachable (${result.latencyMs}ms).`,
          variant: "default",
        });
      } else {
        toast({
          title: "Connection Failed",
          description: result.error || "Unable to connect to provider.",
          variant: "destructive",
        });
      }
    } catch (error) {
      toast({
        title: "Connection Error",
        description: String(error),
        variant: "destructive",
      });
    }
  };

  const handleFormClose = () => {
    setIsFormOpen(false);
    setEditingProvider(undefined);
  };

  const handleAgentFormClose = (isOpen: boolean) => {
    if (!isOpen) {
      setEditingAgentProvider(undefined);
    }
    setIsAgentFormOpen(isOpen);
  };

  const handleAgentEdit = (provider: Provider) => {
    setEditingAgentProvider(provider);
    setIsAgentFormOpen(true);
  };

  const handleAgentUpdate = async (data: CreateProviderInput) => {
    if (!editingAgentProvider) return;
    try {
      const updateData: UpdateProviderInput = {
        name: data.name,
      };
      await updateProvider(editingAgentProvider.id, updateData);
      toast({
        title: "Provider Updated",
        description: `${data.name} has been updated successfully.`,
      });
    } catch (error) {
      toast({
        title: "Error",
        description: String(error),
        variant: "destructive",
      });
      throw error;
    }
  };

  if (isLoading) {
    return (
      <MainContent
        title="Providers"
        description="Add model and agent providers. Use Quota to authenticate and view usage."
      >
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-6 w-6 animate-spin text-primary" />
        </div>
      </MainContent>
    );
  }

  return (
    <MainContent
      title="Providers"
      description="Add model and agent providers. Use Quota to authenticate and view usage."
    >
      {/* Category Toggle */}
      <div className="mb-6 flex items-center justify-between gap-4">
        <Tabs
          value={category}
          onValueChange={(v) => setCategory(v as ProviderCategory)}
        >
          <TabsList>
            <TabsTrigger value="Model">Model</TabsTrigger>
            <TabsTrigger value="Agent">Agent</TabsTrigger>
          </TabsList>
        </Tabs>
      </div>

      {/* Provider Grid */}
      <motion.div
        variants={containerVariants}
        initial="hidden"
        animate="show"
        className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5"
      >
        <AnimatePresence mode="popLayout">
          {filteredProviders.map((provider) => (
            <motion.div key={provider.id} variants={itemVariants} layout initial={false}>
              <ProviderCard
                provider={provider}
                onEdit={provider.category === "Agent" ? handleAgentEdit : handleEdit}
                onTestConnection={handleTestConnection}
              />
            </motion.div>
          ))}
        </AnimatePresence>

        {/* Add Provider Card */}
        <motion.div variants={itemVariants}>
          <button
            onClick={() =>
              category === "Agent"
                ? setIsAgentFormOpen(true)
                : setIsFormOpen(true)
            }
            className="w-full h-full min-h-[180px] rounded-lg border-2 border-dashed border-border hover:border-primary/50 bg-card/50 hover:bg-card transition-all flex flex-col items-center justify-center gap-3 group"
          >
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 group-hover:bg-primary/20 transition-colors">
              <Plus className="h-5 w-5 text-primary" />
            </div>
            <span className="text-sm font-medium text-muted-foreground group-hover:text-foreground transition-colors">
              Add Provider
            </span>
          </button>
        </motion.div>
      </motion.div>

      {/* Provider Form Dialog */}
      <ProviderForm
        open={isFormOpen}
        onOpenChange={handleFormClose}
        provider={editingProvider}
        onSubmit={editingProvider ? handleUpdate : handleCreate}
        onDelete={editingProvider ? handleDelete : undefined}
      />

      {/* Agent Provider Form Dialog */}
      <AgentProviderForm
        open={isAgentFormOpen}
        onOpenChange={handleAgentFormClose}
        provider={editingAgentProvider}
        onSubmit={editingAgentProvider ? handleAgentUpdate : handleCreate}
        onDelete={editingAgentProvider ? handleDelete : undefined}
        existingAgentTypes={existingAgentTypes}
      />
    </MainContent>
  );
}

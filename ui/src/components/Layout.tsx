
import React, { useState } from "react";
import Sidebar from "./Sidebar";
import { Button } from "@/components/ui/button";
import { Settings } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  DialogFooter,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { getServerConfig, saveServerConfig } from "@/utils/api";

interface LayoutProps {
  children: React.ReactNode;
  activeTab: string;
  setActiveTab: (tab: string) => void;
}

const Layout: React.FC<LayoutProps> = ({ children, activeTab, setActiveTab }) => {
  const [serverConfig, setServerConfig] = useState(getServerConfig());
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  const handleSaveConfig = (e: React.FormEvent) => {
    e.preventDefault();
    saveServerConfig(serverConfig.host, Number(serverConfig.port));
    setIsSettingsOpen(false);
    window.location.reload(); // Reload to apply changes
  };

  return (
    <div className="min-h-screen flex flex-col">
      {/* Header */}
      <header className="bg-rustmock-blue text-white p-4 flex justify-between items-center shadow-md">
        <div className="flex items-center space-x-2">
          <h1 className="text-xl font-bold">Rust Mock Dashboard</h1>
        </div>
        <div className="flex items-center space-x-4">
          <div className="text-sm">
            <span className="opacity-70 mr-1">Server:</span>
            <span>{serverConfig.host}:{serverConfig.port}</span>
          </div>
          
          {/* Settings Dialog */}
          <Dialog open={isSettingsOpen} onOpenChange={setIsSettingsOpen}>
            <DialogTrigger asChild>
              <Button variant="ghost" size="icon" className="text-white hover:bg-white/10">
                <Settings className="h-5 w-5" />
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Server Settings</DialogTitle>
              </DialogHeader>
              <form onSubmit={handleSaveConfig} className="space-y-4 pt-4">
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="host">Host</Label>
                    <Input
                      id="host"
                      value={serverConfig.host}
                      onChange={(e) => setServerConfig({ ...serverConfig, host: e.target.value })}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="port">Port</Label>
                    <Input
                      id="port"
                      type="number"
                      value={serverConfig.port}
                      onChange={(e) => setServerConfig({ ...serverConfig, port: parseInt(e.target.value) })}
                    />
                  </div>
                </div>
                <DialogFooter>
                  <Button type="submit">Save Changes</Button>
                </DialogFooter>
              </form>
            </DialogContent>
          </Dialog>
        </div>
      </header>

      {/* Main content */}
      <div className="flex-1 flex">
        <Sidebar activeTab={activeTab} setActiveTab={setActiveTab} />
        <main className="flex-1 overflow-auto p-6 bg-gray-50">
          {children}
        </main>
      </div>

      {/* Footer */}
      <footer className="bg-rustmock-blue text-white py-2 px-4 text-sm text-center">
        <p>Rust Mock Dashboard &copy; {new Date().getFullYear()}</p>
      </footer>
    </div>
  );
};

export default Layout;

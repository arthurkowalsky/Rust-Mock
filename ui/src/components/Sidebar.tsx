
import React from "react";
import { Activity, Plus, List, Play } from "lucide-react";
import { cn } from "@/lib/utils";

interface SidebarProps {
  activeTab: string;
  setActiveTab: (tab: string) => void;
}

type NavItem = {
  id: string;
  label: string;
  icon: React.ComponentType<any>;
};

const navItems: NavItem[] = [
  {
    id: "endpoints",
    label: "Endpoints",
    icon: List,
  },
  {
    id: "logs",
    label: "Logs",
    icon: Activity,
  },
  {
    id: "add-endpoint",
    label: "Add Endpoint",
    icon: Plus,
  },
  {
    id: "test-endpoint",
    label: "Test Endpoint",
    icon: Play,
  },
];

const Sidebar: React.FC<SidebarProps> = ({ activeTab, setActiveTab }) => {
  return (
    <aside className="w-64 bg-rustmock-blue text-white border-r border-sidebar-border shrink-0 hidden md:block">
      <nav className="p-4 space-y-1">
        {navItems.map((item) => (
          <button
            key={item.id}
            onClick={() => setActiveTab(item.id)}
            className={cn(
              "w-full flex items-center space-x-3 px-4 py-3 rounded-md transition-colors",
              activeTab === item.id
                ? "bg-rustmock-orange text-white"
                : "text-gray-200 hover:bg-white/10"
            )}
          >
            <item.icon className="h-5 w-5" />
            <span>{item.label}</span>
          </button>
        ))}
      </nav>
    </aside>
  );
};

export default Sidebar;

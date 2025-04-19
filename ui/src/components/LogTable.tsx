
import React, { useState } from "react";
import { RequestLog, HttpMethod } from "@/types";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Eye, Trash2, RefreshCw } from "lucide-react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import JsonEditor from "./JsonEditor";
import { toast } from "sonner";

interface LogTableProps {
  logs: RequestLog[];
  onRefresh: () => void;
}

const LogTable: React.FC<LogTableProps> = ({ logs, onRefresh }) => {
  const [filter, setFilter] = useState({
    method: "all",
    path: "",
    status: "all",
  });
  
  const [detailsLog, setDetailsLog] = useState<RequestLog | null>(null);

  // Handle filtering logs
  const filteredLogs = logs.filter((log) => {
    const methodMatch = filter.method === "all" || log.method === filter.method;
    const pathMatch = filter.path
      ? log.path.toLowerCase().includes(filter.path.toLowerCase())
      : true;
    const statusMatch = filter.status === "all"
      ? true
      : log.status.toString().startsWith(filter.status);
    return methodMatch && pathMatch && statusMatch;
  });

  // Handle clearing logs
  const handleClearLogs = async () => {
    const confirmed = window.confirm(
      "Are you sure you want to clear all logs?"
    );
    if (confirmed) {
      try {
        const response = await fetch(`${window.location.origin}/__mock/logs`, {
          method: 'DELETE'
        });
        
        if (response.ok) {
          onRefresh(); // Refresh the logs list
          toast.success("Logs cleared successfully");
        } else {
          toast.error("Failed to clear logs");
        }
      } catch (error) {
        console.error("Error clearing logs:", error);
        toast.error("Failed to clear logs");
      }
    }
  };

  // Format the timestamp
  const formatTimestamp = (timestamp: string) => {
    try {
      const date = new Date(timestamp);
      return date.toLocaleString();
    } catch (error) {
      return timestamp;
    }
  };

  // Reverse the logs array to show newest first
  const sortedLogs = [...filteredLogs].reverse();

  return (
    <div className="space-y-4">
      {/* Filters */}
      <div className="flex flex-col sm:flex-row gap-4">
        <div className="sm:w-1/4">
          <Select
            value={filter.method}
            onValueChange={(value) => setFilter({ ...filter, method: value })}
          >
            <SelectTrigger>
              <SelectValue placeholder="Method" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Methods</SelectItem>
              <SelectItem value="GET">GET</SelectItem>
              <SelectItem value="POST">POST</SelectItem>
              <SelectItem value="PUT">PUT</SelectItem>
              <SelectItem value="PATCH">PATCH</SelectItem>
              <SelectItem value="DELETE">DELETE</SelectItem>
            </SelectContent>
          </Select>
        </div>
        
        <div className="sm:w-1/4">
          <Select
            value={filter.status}
            onValueChange={(value) => setFilter({ ...filter, status: value })}
          >
            <SelectTrigger>
              <SelectValue placeholder="Status" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Status</SelectItem>
              <SelectItem value="2">2xx (Success)</SelectItem>
              <SelectItem value="3">3xx (Redirect)</SelectItem>
              <SelectItem value="4">4xx (Client Error)</SelectItem>
              <SelectItem value="5">5xx (Server Error)</SelectItem>
            </SelectContent>
          </Select>
        </div>
        
        <div className="flex-1">
          <Input
            placeholder="Filter by path"
            value={filter.path}
            onChange={(e) => setFilter({ ...filter, path: e.target.value })}
          />
        </div>
        
        <div className="flex gap-2">
          <Button onClick={onRefresh} variant="outline" size="icon">
            <RefreshCw className="h-4 w-4" />
          </Button>
          <Button
            onClick={handleClearLogs}
            variant="outline"
            className="text-red-500 border-red-200 hover:bg-red-50"
          >
            <Trash2 className="h-4 w-4 mr-2" />
            Clear Logs
          </Button>
        </div>
      </div>

      {/* Logs Table */}
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-40">Timestamp</TableHead>
              <TableHead className="w-24">Method</TableHead>
              <TableHead>Path</TableHead>
              <TableHead className="w-24">Status</TableHead>
              <TableHead className="text-right w-20">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {sortedLogs.length > 0 ? (
              sortedLogs.map((log, index) => (
                <TableRow key={`${log.timestamp}-${index}`}>
                  <TableCell className="text-xs text-gray-500">
                    {formatTimestamp(log.timestamp)}
                  </TableCell>
                  <TableCell>
                    <span
                      className={`px-2 py-1 rounded text-xs font-medium method-${log.method.toLowerCase()}`}
                    >
                      {log.method}
                    </span>
                  </TableCell>
                  <TableCell className="font-mono text-sm truncate max-w-[300px]">
                    {log.path}
                  </TableCell>
                  <TableCell>
                    <span
                      className={`px-2 py-1 rounded text-xs font-medium ${
                        log.status >= 200 && log.status < 300
                          ? "bg-green-100 text-green-800"
                          : log.status >= 300 && log.status < 400
                          ? "bg-blue-100 text-blue-800"
                          : log.status >= 400 && log.status < 500
                          ? "bg-yellow-100 text-yellow-800"
                          : "bg-red-100 text-red-800"
                      }`}
                    >
                      {log.status}
                    </span>
                  </TableCell>
                  <TableCell className="text-right">
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => setDetailsLog(log)}
                    >
                      <Eye className="h-4 w-4" />
                    </Button>
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell colSpan={5} className="text-center py-8 text-gray-500">
                  No logs found
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>

      {/* Log Details Dialog */}
      <Dialog
        open={!!detailsLog}
        onOpenChange={(open) => !open && setDetailsLog(null)}
      >
        <DialogContent className="max-w-3xl max-h-[90vh] overflow-auto">
          <DialogHeader>
            <DialogTitle>
              {detailsLog?.method} {detailsLog?.path}
            </DialogTitle>
          </DialogHeader>

          {detailsLog && (
            <div className="space-y-4 mt-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <h3 className="text-sm font-medium mb-2">Timestamp</h3>
                  <div className="bg-gray-100 p-2 rounded">
                    {formatTimestamp(detailsLog.timestamp)}
                  </div>
                </div>
                <div>
                  <h3 className="text-sm font-medium mb-2">Status</h3>
                  <div className="bg-gray-100 p-2 rounded">{detailsLog.status}</div>
                </div>
              </div>

              <div>
                <h3 className="text-sm font-medium mb-2">Headers</h3>
                <div className="bg-gray-100 p-2 rounded font-mono text-sm max-h-40 overflow-auto">
                  {Object.entries(detailsLog.headers).map(([key, value]) => (
                    <div key={key}>
                      <span className="text-purple-600">{key}</span>:{" "}
                      <span>{value}</span>
                    </div>
                  ))}
                </div>
              </div>

              {detailsLog.query && Object.keys(detailsLog.query).length > 0 && (
                <div>
                  <h3 className="text-sm font-medium mb-2">Query Parameters</h3>
                  <div className="bg-gray-100 p-2 rounded font-mono text-sm">
                    {Object.entries(detailsLog.query).map(([key, value]) => (
                      <div key={key}>
                        <span className="text-blue-600">{key}</span>:{" "}
                        <span>{value}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {detailsLog.body && (
                <div>
                  <h3 className="text-sm font-medium mb-2">Request Body</h3>
                  <JsonEditor value={detailsLog.body} onChange={() => {}} />
                </div>
              )}
            </div>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
};

export default LogTable;

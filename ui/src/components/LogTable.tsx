
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
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Eye, Trash2, RefreshCw, ArrowRightLeft } from "lucide-react";
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

  const formatTimestamp = (timestamp: string) => {
    try {
      const date = new Date(timestamp);
      return date.toLocaleString();
    } catch (error) {
      return timestamp;
    }
  };

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
              <TableHead className="w-32">Source</TableHead>
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
                    {log.proxied_to ? (
                      <Badge variant="outline" className="bg-purple-50 text-purple-700 border-purple-200 flex items-center gap-1 w-fit">
                        <ArrowRightLeft className="w-3 h-3" />
                        Proxied
                      </Badge>
                    ) : (
                      <Badge variant="outline" className="bg-blue-50 text-blue-700 border-blue-200">
                        Mock
                      </Badge>
                    )}
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
                <TableCell colSpan={6} className="text-center py-8 text-gray-500">
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
        <DialogContent className="max-w-6xl max-h-[90vh] overflow-auto">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-3">
              <span
                className={`px-2 py-1 rounded text-xs font-medium ${
                  detailsLog && detailsLog.status >= 200 && detailsLog.status < 300
                    ? "bg-green-100 text-green-800"
                    : detailsLog && detailsLog.status >= 400 && detailsLog.status < 500
                    ? "bg-yellow-100 text-yellow-800"
                    : "bg-red-100 text-red-800"
                }`}
              >
                {detailsLog?.status}
              </span>
              <span>{detailsLog?.method} {detailsLog?.path}</span>
            </DialogTitle>
          </DialogHeader>

          {detailsLog && (
            <div className="space-y-4 mt-4">
              {/* Metadata Row */}
              <div className="grid grid-cols-3 gap-4 pb-4 border-b">
                <div>
                  <h3 className="text-xs font-medium text-gray-500 mb-1">Timestamp</h3>
                  <div className="text-sm">{formatTimestamp(detailsLog.timestamp)}</div>
                </div>
                {detailsLog.matched_endpoint && (
                  <div>
                    <h3 className="text-xs font-medium text-gray-500 mb-1">Matched Endpoint</h3>
                    <div className="text-sm font-mono bg-blue-50 px-2 py-1 rounded inline-block">
                      {detailsLog.matched_endpoint}
                    </div>
                  </div>
                )}
                {detailsLog.proxied_to && (
                  <div>
                    <h3 className="text-xs font-medium text-gray-500 mb-1">Proxied To</h3>
                    <div className="text-xs font-mono bg-purple-50 px-2 py-1 rounded break-all">
                      {detailsLog.proxied_to}
                    </div>
                  </div>
                )}
              </div>

              {/* Request/Response Split View */}
              <div className="grid grid-cols-2 gap-4">
                {/* REQUEST COLUMN */}
                <div className="space-y-4 border-r pr-4">
                  <h2 className="text-lg font-semibold text-blue-600">📨 Request</h2>

                  {/* Request Headers */}
                  <div>
                    <h3 className="text-sm font-medium mb-2">Headers</h3>
                    <div className="bg-gray-50 p-3 rounded font-mono text-xs max-h-40 overflow-auto border">
                      {Object.keys(detailsLog.request_headers).length > 0 ? (
                        Object.entries(detailsLog.request_headers).map(([key, value]) => (
                          <div key={key} className="mb-1">
                            <span className="text-purple-600 font-medium">{key}</span>:{" "}
                            <span className="text-gray-700">{value}</span>
                          </div>
                        ))
                      ) : (
                        <div className="text-gray-500">No headers</div>
                      )}
                    </div>
                  </div>

                  {/* Query Parameters */}
                  {detailsLog.query && (
                    <div>
                      <h3 className="text-sm font-medium mb-2">Query String</h3>
                      <div className="bg-gray-50 p-3 rounded font-mono text-xs border">
                        {detailsLog.query || <span className="text-gray-500">No query parameters</span>}
                      </div>
                    </div>
                  )}

                  {/* Request Body */}
                  <div>
                    <h3 className="text-sm font-medium mb-2">Body</h3>
                    {detailsLog.request_body ? (
                      <JsonEditor value={detailsLog.request_body} onChange={() => {}} />
                    ) : (
                      <div className="bg-gray-50 p-3 rounded text-xs text-gray-500 border">
                        No request body
                      </div>
                    )}
                  </div>
                </div>

                {/* RESPONSE COLUMN */}
                <div className="space-y-4 pl-4">
                  <h2 className="text-lg font-semibold text-green-600">📤 Response</h2>

                  {/* Response Status */}
                  <div>
                    <h3 className="text-sm font-medium mb-2">Status Code</h3>
                    <div className="bg-gray-50 p-3 rounded border">
                      <span
                        className={`px-3 py-1 rounded text-sm font-medium ${
                          detailsLog.status >= 200 && detailsLog.status < 300
                            ? "bg-green-100 text-green-800"
                            : detailsLog.status >= 400 && detailsLog.status < 500
                            ? "bg-yellow-100 text-yellow-800"
                            : "bg-red-100 text-red-800"
                        }`}
                      >
                        {detailsLog.status} {
                          detailsLog.status === 200 ? "OK" :
                          detailsLog.status === 201 ? "Created" :
                          detailsLog.status === 204 ? "No Content" :
                          detailsLog.status === 404 ? "Not Found" : ""
                        }
                      </span>
                    </div>
                  </div>

                  {/* Response Headers */}
                  <div>
                    <h3 className="text-sm font-medium mb-2">Headers</h3>
                    <div className="bg-gray-50 p-3 rounded font-mono text-xs max-h-40 overflow-auto border">
                      {Object.keys(detailsLog.response_headers).length > 0 ? (
                        Object.entries(detailsLog.response_headers).map(([key, value]) => (
                          <div key={key} className="mb-1">
                            <span className="text-green-600 font-medium">{key}</span>:{" "}
                            <span className="text-gray-700">{value}</span>
                          </div>
                        ))
                      ) : (
                        <div className="text-gray-500">No headers</div>
                      )}
                    </div>
                  </div>

                  {/* Response Body */}
                  <div>
                    <h3 className="text-sm font-medium mb-2">Body</h3>
                    {detailsLog.response_body ? (
                      <JsonEditor value={detailsLog.response_body} onChange={() => {}} />
                    ) : (
                      <div className="bg-gray-50 p-3 rounded text-xs text-gray-500 border">
                        No response body
                      </div>
                    )}
                  </div>
                </div>
              </div>
            </div>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
};

export default LogTable;

import React, { useState } from "react";
import { Endpoint, HttpMethod } from "@/types";
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
import { Eye, Play, Trash2 } from "lucide-react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import JsonEditor from "./JsonEditor";
import { removeEndpoint } from "@/utils/api";
import { toast } from "sonner";

interface EndpointTableProps {
  endpoints: Endpoint[];
  onRemove: () => void;
  onTest: (endpoint: Endpoint) => void;
}

const EndpointTable: React.FC<EndpointTableProps> = ({
  endpoints,
  onRemove,
  onTest,
}) => {
  const [filter, setFilter] = useState({ method: "all", path: "" });
  const [detailsEndpoint, setDetailsEndpoint] = useState<Endpoint | null>(null);

  const filteredEndpoints = endpoints.filter((endpoint) => {
    const methodMatch = filter.method === "all" || endpoint.method === filter.method;
    const pathMatch = filter.path
      ? endpoint.path.toLowerCase().includes(filter.path.toLowerCase())
      : true;
    return methodMatch && pathMatch;
  });

  const handleRemove = async (endpoint: Endpoint) => {
    const confirmed = window.confirm(
      `Are you sure you want to remove the endpoint ${endpoint.method} ${endpoint.path}?`
    );
    if (confirmed) {
      const result = await removeEndpoint(endpoint.method, endpoint.path);
      if (!result.error) {
        onRemove();
      }
    }
  };

  return (
    <div className="space-y-4">
      {/* Filters */}
      <div className="flex flex-col sm:flex-row gap-4">
        <div className="sm:w-1/3">
          <Select
            value={filter.method}
            onValueChange={(value) => setFilter({ ...filter, method: value })}
          >
            <SelectTrigger>
              <SelectValue placeholder="Filter by method" />
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
        <div className="flex-1">
          <Input
            placeholder="Filter by path"
            value={filter.path}
            onChange={(e) => setFilter({ ...filter, path: e.target.value })}
          />
        </div>
      </div>

      {/* Endpoints Table */}
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-24">Method</TableHead>
              <TableHead>Path</TableHead>
              <TableHead className="w-24">Status</TableHead>
              <TableHead className="text-right w-32">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {filteredEndpoints.length > 0 ? (
              filteredEndpoints.map((endpoint, index) => (
                <TableRow key={`${endpoint.method}-${endpoint.path}-${index}`}>
                  <TableCell>
                    <span
                      className={`px-2 py-1 rounded text-xs font-medium method-${endpoint.method.toLowerCase()}`}
                    >
                      {endpoint.method}
                    </span>
                  </TableCell>
                  <TableCell className="font-mono text-sm">
                    {endpoint.path}
                  </TableCell>
                  <TableCell>{endpoint.status || 200}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex justify-end gap-2">
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => setDetailsEndpoint(endpoint)}
                      >
                        <Eye className="h-4 w-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => onTest(endpoint)}
                      >
                        <Play className="h-4 w-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleRemove(endpoint)}
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell colSpan={4} className="text-center py-8 text-gray-500">
                  No endpoints found
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>

      {/* Endpoint Details Dialog */}
      <Dialog
        open={!!detailsEndpoint}
        onOpenChange={(open) => !open && setDetailsEndpoint(null)}
      >
        <DialogContent className="max-w-3xl max-h-[90vh] overflow-auto">
          <DialogHeader>
            <DialogTitle>
              {detailsEndpoint?.method} {detailsEndpoint?.path}
            </DialogTitle>
          </DialogHeader>

          {detailsEndpoint && (
            <div className="space-y-4 mt-4">
              <div>
                <h3 className="text-sm font-medium mb-2">Response Status</h3>
                <div className="bg-gray-100 p-2 rounded">{detailsEndpoint.status || 200}</div>
              </div>

              <div>
                <h3 className="text-sm font-medium mb-2">Response Headers</h3>
                {detailsEndpoint.headers && Object.keys(detailsEndpoint.headers).length > 0 ? (
                  <div className="bg-gray-100 p-2 rounded font-mono text-sm">
                    {Object.entries(detailsEndpoint.headers).map(([key, value]) => (
                      <div key={key}>
                        <span className="text-purple-600">{key}</span>:{" "}
                        <span>{value}</span>
                      </div>
                    ))}
                  </div>
                ) : (
                  <div className="bg-gray-100 p-2 rounded text-gray-500">No custom headers</div>
                )}
              </div>

              <div>
                <h3 className="text-sm font-medium mb-2">Response Body</h3>
                <JsonEditor value={detailsEndpoint.response} onChange={() => {}} />
              </div>
            </div>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
};

export default EndpointTable;

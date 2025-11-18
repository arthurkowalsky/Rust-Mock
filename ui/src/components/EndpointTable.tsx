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
import { Eye, Play, Trash2, Pencil } from "lucide-react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import JsonEditor from "./JsonEditor";
import KeyValueEditor from "./KeyValueEditor";
import { removeEndpoint, updateEndpoint } from "@/utils/api";
import { toast } from "sonner";
import { Label } from "@/components/ui/label";
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs";
import { KeyValuePair } from "@/types";

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
  const [editEndpoint, setEditEndpoint] = useState<Endpoint | null>(null);
  const [editForm, setEditForm] = useState<Endpoint | null>(null);
  const [editHeaderPairs, setEditHeaderPairs] = useState<KeyValuePair[]>([]);

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

  const handleEditClick = (endpoint: Endpoint) => {
    setEditEndpoint(endpoint);
    setEditForm({ ...endpoint });
    const pairs = endpoint.headers
      ? Object.entries(endpoint.headers).map(([key, value]) => ({ key, value }))
      : [];
    setEditHeaderPairs(pairs);
  };

  const handleEditSave = async () => {
    if (!editEndpoint || !editForm) return;

    if (!editForm.path.startsWith("/")) {
      toast.error("Path must start with '/'");
      return;
    }

    const result = await updateEndpoint(
      editEndpoint.method,
      editEndpoint.path,
      editForm
    );

    if (result.updated) {
      setEditEndpoint(null);
      setEditForm(null);
      setEditHeaderPairs([]);
      onRemove();
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
                        onClick={() => handleEditClick(endpoint)}
                      >
                        <Pencil className="h-4 w-4" />
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

      <Dialog
        open={!!editEndpoint}
        onOpenChange={(open) => {
          if (!open) {
            setEditEndpoint(null);
            setEditForm(null);
            setEditHeaderPairs([]);
          }
        }}
      >
        <DialogContent className="max-w-4xl max-h-[90vh] overflow-auto">
          <DialogHeader>
            <DialogTitle>
              Edit Endpoint: {editEndpoint?.method} {editEndpoint?.path}
            </DialogTitle>
          </DialogHeader>

          {editForm && (
            <form
              onSubmit={(e) => {
                e.preventDefault();
                handleEditSave();
              }}
              className="space-y-4"
            >
              <div className="grid grid-cols-3 gap-4">
                <div className="space-y-2">
                  <Label>HTTP Method</Label>
                  <Select
                    value={editForm.method}
                    onValueChange={(value) =>
                      setEditForm({ ...editForm, method: value as HttpMethod })
                    }
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="GET">GET</SelectItem>
                      <SelectItem value="POST">POST</SelectItem>
                      <SelectItem value="PUT">PUT</SelectItem>
                      <SelectItem value="PATCH">PATCH</SelectItem>
                      <SelectItem value="DELETE">DELETE</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-2">
                  <Label>Path</Label>
                  <Input
                    value={editForm.path}
                    onChange={(e) =>
                      setEditForm({ ...editForm, path: e.target.value })
                    }
                    placeholder="/api/users"
                  />
                </div>

                <div className="space-y-2">
                  <Label>Status Code</Label>
                  <Input
                    type="number"
                    value={editForm.status}
                    onChange={(e) =>
                      setEditForm({ ...editForm, status: Number(e.target.value) })
                    }
                  />
                </div>
              </div>

              <Tabs defaultValue="response">
                <TabsList>
                  <TabsTrigger value="response">Response</TabsTrigger>
                  <TabsTrigger value="headers">Headers</TabsTrigger>
                  <TabsTrigger value="proxy">Proxy</TabsTrigger>
                </TabsList>

                <TabsContent value="response" className="pt-4">
                  <Label>Response Body (JSON)</Label>
                  <JsonEditor
                    value={editForm.response}
                    onChange={(value) =>
                      setEditForm({ ...editForm, response: value })
                    }
                  />
                </TabsContent>

                <TabsContent value="headers" className="pt-4">
                  <Label>Response Headers</Label>
                  <KeyValueEditor
                    pairs={editHeaderPairs}
                    onChange={(pairs) => {
                      setEditHeaderPairs(pairs);
                      const headersObj: Record<string, string> = {};
                      pairs.forEach((pair) => {
                        if (pair.key && pair.key.trim()) {
                          headersObj[pair.key.trim()] = pair.value;
                        }
                      });
                      setEditForm({ ...editForm, headers: headersObj });
                    }}
                    placeholder={{ key: "Header", value: "Value" }}
                  />
                </TabsContent>

                <TabsContent value="proxy" className="pt-4">
                  <Label>Proxy URL (optional)</Label>
                  <Input
                    value={editForm.proxy_url || ""}
                    onChange={(e) =>
                      setEditForm({
                        ...editForm,
                        proxy_url: e.target.value.trim() || undefined,
                      })
                    }
                    placeholder="https://api.example.com"
                  />
                  <p className="text-sm text-muted-foreground mt-2">
                    If set, requests will be forwarded to this URL
                  </p>
                </TabsContent>
              </Tabs>

              <div className="flex justify-end gap-2">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => {
                    setEditEndpoint(null);
                    setEditForm(null);
                    setEditHeaderPairs([]);
                  }}
                >
                  Cancel
                </Button>
                <Button type="submit">Save Changes</Button>
              </div>
            </form>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
};

export default EndpointTable;


import React, { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { KeyValuePair } from "@/types";
import { X, Plus } from "lucide-react";

interface KeyValueEditorProps {
  pairs: KeyValuePair[];
  onChange: (pairs: KeyValuePair[]) => void;
  placeholder?: {
    key: string;
    value: string;
  };
  allowEmpty?: boolean;
}

const KeyValueEditor: React.FC<KeyValueEditorProps> = ({
  pairs,
  onChange,
  placeholder = { key: "Key", value: "Value" },
  allowEmpty = true,
}) => {
  const [items, setItems] = useState<KeyValuePair[]>(
    pairs.length > 0 ? pairs : [{ key: "", value: "" }]
  );

  useEffect(() => {
    if (pairs.length > 0 && JSON.stringify(pairs) !== JSON.stringify(items)) {
      setItems(pairs);
    }
  }, [pairs]);

  const handleChange = (index: number, field: "key" | "value", value: string) => {
    const updatedItems = [...items];
    updatedItems[index] = { ...updatedItems[index], [field]: value };
    setItems(updatedItems);
    onChange(updatedItems.filter(item => allowEmpty || (item.key !== "" || item.value !== "")));
  };

  const handleAddPair = () => {
    const updatedItems = [...items, { key: "", value: "" }];
    setItems(updatedItems);
  };

  const handleRemovePair = (index: number) => {
    if (items.length === 1) {
      setItems([{ key: "", value: "" }]);
      onChange([]);
      return;
    }

    const updatedItems = items.filter((_, i) => i !== index);
    setItems(updatedItems);
    onChange(updatedItems.filter(item => allowEmpty || (item.key !== "" || item.value !== "")));
  };

  return (
    <div className="space-y-2">
      {items.map((pair, index) => (
        <div key={index} className="flex items-center gap-2">
          <Input
            value={pair.key}
            onChange={(e) => handleChange(index, "key", e.target.value)}
            placeholder={placeholder.key}
            className="flex-1"
          />
          <Input
            value={pair.value}
            onChange={(e) => handleChange(index, "value", e.target.value)}
            placeholder={placeholder.value}
            className="flex-1"
          />
          <Button
            type="button"
            variant="ghost"
            size="icon"
            onClick={() => handleRemovePair(index)}
            className="flex-shrink-0"
          >
            <X className="h-4 w-4" />
          </Button>
        </div>
      ))}
      <Button
        type="button"
        variant="outline"
        size="sm"
        onClick={handleAddPair}
        className="w-full"
      >
        <Plus className="h-4 w-4 mr-2" />
        Add {placeholder.key}
      </Button>
    </div>
  );
};

export default KeyValueEditor;

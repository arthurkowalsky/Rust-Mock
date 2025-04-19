
import React, { useState, useEffect } from "react";
import { Textarea } from "@/components/ui/textarea";
import { toast } from "sonner";

interface JsonEditorProps {
  value: any;
  onChange: (value: any) => void;
  placeholder?: string;
  height?: string;
}

const JsonEditor: React.FC<JsonEditorProps> = ({
  value,
  onChange,
  placeholder = "Enter JSON...",
  height = "h-64"
}) => {
  const [text, setText] = useState<string>("");
  const [isValid, setIsValid] = useState<boolean>(true);

  // Format the JSON string
  const formatJson = (json: string): string => {
    try {
      const obj = JSON.parse(json);
      return JSON.stringify(obj, null, 2);
    } catch (e) {
      return json;
    }
  };

  // Initialize the editor with the provided value
  useEffect(() => {
    if (value) {
      const jsonString = typeof value === 'string' 
        ? value 
        : JSON.stringify(value, null, 2);
      setText(jsonString);
    }
  }, [value]);

  // Handle changes to the text
  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const newText = e.target.value;
    setText(newText);
    
    try {
      // Only parse if there's text
      if (newText.trim()) {
        const parsed = JSON.parse(newText);
        onChange(parsed);
        setIsValid(true);
      } else {
        onChange({});
        setIsValid(true);
      }
    } catch (error) {
      setIsValid(false);
    }
  };

  // Format the JSON when the editor loses focus
  const handleBlur = () => {
    try {
      if (text.trim()) {
        const formatted = formatJson(text);
        setText(formatted);
        setIsValid(true);
      }
    } catch (error) {
      setIsValid(false);
      toast.error("Invalid JSON");
    }
  };

  return (
    <div className="relative">
      <Textarea
        value={text}
        onChange={handleChange}
        onBlur={handleBlur}
        placeholder={placeholder}
        className={`font-mono text-sm ${height} ${!isValid ? 'border-red-500' : ''}`}
      />
      {!isValid && (
        <div className="text-red-500 text-xs mt-1">
          Invalid JSON format
        </div>
      )}
    </div>
  );
};

export default JsonEditor;

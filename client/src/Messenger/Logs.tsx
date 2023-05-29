import { Stack, Box, Typography, Avatar, useTheme } from "@mui/material";
import { useEffect, useState } from "react";
import { emit, listen } from "@tauri-apps/api/event";
import HubIcon from "@mui/icons-material/Hub";

export default function Logs() {
  const theme = useTheme();
  const [logs, setLogs] = useState<String[]>([]);

  useEffect(() => {
    listen<String>("tor-change", (event) => {
      setLogs((prev) => [...prev, event.payload]);
    });
  }, []);

  return (
    <>
      <Box
        padding={2.5}
        sx={{ border: `${theme.colors.alpha.black[50]} solid 2px` }}
      >
        <Stack direction={"row"} gap={1}>
          <Box display={"flex"} alignItems={"center"}>
            <HubIcon sx={{ mr: 1 }} />
            <Typography variant="h5" noWrap>
              Logs
            </Typography>
          </Box>
        </Stack>
      </Box>
      <Box>
        {logs.map((log) => (
          <Typography key={Math.random()} m={0.3}>
            {log}
          </Typography>
        ))}
      </Box>
    </>
  );
}

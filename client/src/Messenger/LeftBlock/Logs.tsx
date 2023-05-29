import { Stack, Box, Typography, useTheme } from "@mui/material";
import { useEffect, useState } from "react";
import { emit, listen } from "@tauri-apps/api/event";
import HubIcon from "@mui/icons-material/Hub";
import Scrollbars from "react-custom-scrollbars-2";

export default function Logs() {
  const theme = useTheme();
  const [logs, setLogs] = useState<String[]>([]);

  useEffect(() => {
    listen<String>("tor-change-logs", (event) => {
      setLogs((prev) => [...prev, event.payload]);
    });
  }, []);

  return (
    <Box>
      <Box
        padding={3.345}
        sx={{ borderBottom: `${theme.colors.alpha.black[50]} solid 2px` }}
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
      {logs.length > 0 && (
        <Box height={"100vh"}>
          <Scrollbars>
            <Stack display={"flex"} justifyContent={"start"}>
              {logs.map((log) => (
                <Typography key={Math.random()} m={0.3} variant="body1">
                  {log}
                </Typography>
              ))}
            </Stack>
          </Scrollbars>
        </Box>
      )}
    </Box>
  );
}

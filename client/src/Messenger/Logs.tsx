import { Stack, Box, Typography, Avatar, useTheme } from "@mui/material";
import { useEffect, useState } from "react";
import { emit, listen } from "@tauri-apps/api/event";

export default function Logs() {
  const theme = useTheme();
  const [logs, setLogs] = useState<String[]>([]);

  useEffect(() => {
    listen<String>("tor-change", (event) => {
      console.log(event);
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
          <Avatar />
          <Box display={"flex"} alignItems={"center"}>
            <Typography variant="h5" noWrap>
              {"user.name"}
            </Typography>
          </Box>
        </Stack>
      </Box>
      <Box>
        <ul>
          {logs.map((log) => (
            <li>{log}</li>
          ))}
        </ul>
      </Box>
    </>
  );
}

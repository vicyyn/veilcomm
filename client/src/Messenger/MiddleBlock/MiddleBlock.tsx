import { Box, Button, Stack, TextField, useTheme } from "@mui/material";
import TopBarContent from "./TopBarContent";
import ChatContent from "./ChatContent";
import BottomBarContent from "./BottomBarContent";
import { useState } from "react";
import { listen, emit } from "@tauri-apps/api/event";
import { LoadingButton } from "@mui/lab";

export default function MiddleBlock() {
  const theme = useTheme();
  const [connected, setConnected] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [clientId, setClientId] = useState("");

  const connect = () => {
    setIsLoading(true);
    emit("tor-event-connect", clientId);

    listen("tor-change-connected", () => {
      setIsLoading(false);
      setConnected(true);
    });
  };

  if (connected) {
    return (
      <Box
        sx={{
          display: "flex",
          flexDirection: "column",
          height: "100vh",
        }}
      >
        <Box
          sx={{
            padding: `${theme.spacing(2)}`,
            borderTop: `${theme.colors.alpha.black[50]} solid 2px`,
            borderBottom: `${theme.colors.alpha.black[50]} solid 2px`,
            alignItems: "center",
          }}
        >
          <TopBarContent />
        </Box>
        <ChatContent />
        <BottomBarContent />
      </Box>
    );
  }

  return (
    <Stack
      m={1}
      gap={2}
      direction={"row"}
      sx={{
        display: "flex",
        height: "100vh",
        justifyContent: "center",
        alignItems: "center",
      }}
    >
      <TextField
        variant="outlined"
        size="small"
        value={clientId}
        onChange={(e) => setClientId(e.target.value)}
        inputProps={{ style: { color: "white", fontSize: "1rem" } }}
        focused={true}
        placeholder="Write your destination key here"
        fullWidth
      />
      <LoadingButton variant="contained" loading={isLoading} onClick={connect}>
        Connect
      </LoadingButton>
    </Stack>
  );
}

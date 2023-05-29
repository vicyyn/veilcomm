import {
  Box,
  Card,
  styled,
  Stack,
  TextField,
  useTheme,
  CircularProgress,
} from "@mui/material";
import TopBarContent from "./TopBarContent";
import ChatContent from "./ChatContent";
import BottomBarContent from "./BottomBarContent";
import { ReactNode, useEffect, useState } from "react";
import { listen, emit } from "@tauri-apps/api/event";
import { LoadingButton } from "@mui/lab";
import Scrollbars from "react-custom-scrollbars-2";

const CardWrapperPrimary = styled(Card)(
  ({ theme }) => `
      background: ${theme.colors.primary.main};
      color: ${theme.palette.primary.contrastText};
      padding: ${theme.spacing(2)};
      border-radius: ${theme.general.borderRadiusXl};
      border-top-right-radius: ${theme.general.borderRadius};
      max-width: 380px;
      display: inline-flex;
`
);

const CardWrapperSecondary = styled(Card)(
  ({ theme }) => `
      background: ${theme.palette.grey[200]};
      color: ${theme.colors.alpha.black[100]};
      padding: ${theme.spacing(2)};
      border-radius: ${theme.general.borderRadiusXl};
      border-top-left-radius: ${theme.general.borderRadius};
      max-width: 380px;
      display: inline-flex;
`
);

export default function MiddleBlock() {
  const theme = useTheme();
  const [connected, setConnected] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [clientId, setClientId] = useState("");
  const [conversation, setConversation] = useState<ReactNode[]>([]);

  useEffect(() => {
    listen<string>("tor-event-receive-message", (event) =>
      receiveMessage(event.payload)
    );
  }, []);

  const VielCommLoadingButton = styled(LoadingButton)(({ theme }) => ({
    "&.Mui-disabled": {
      color: "grey",
    },
  }));

  const connect = () => {
    setIsLoading(true);
    emit("tor-event-connect", clientId);

    listen("tor-change-connected", () => {
      setIsLoading(false);
      setConnected(true);
    });
  };

  const sendMessage = (text: string) => {
    setConversation((prev) => [...prev, renderRight(text)]);
  };

  const receiveMessage = (text: string) => {
    setConversation((prev) => [...prev, renderLeft(text)]);
  };

  const renderLeft = (text: string) => (
    <Box
      key={Math.random()}
      display="flex"
      alignItems="flex-start"
      justifyContent="flex-start"
      py={1}
    >
      <CardWrapperSecondary>{text}</CardWrapperSecondary>
    </Box>
  );

  const renderRight = (text: string) => (
    <Box
      key={Math.random()}
      display="flex"
      alignItems="flex-start"
      justifyContent="flex-end"
      py={1}
    >
      <CardWrapperPrimary>{text}</CardWrapperPrimary>
    </Box>
  );

  const endConversation = () => {
    setConversation([]);
    setClientId("");
    setConnected(false);
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
          <TopBarContent
            clientId={clientId}
            endConversation={endConversation}
          />
        </Box>
        <Scrollbars>
          {conversation.length > 0 ? (
            conversation
          ) : (
            <Box
              height={"70vh"}
              display={"flex"}
              justifyContent={"center"}
              alignItems={"center"}
            >
              Start your conversation with {clientId}
            </Box>
          )}
        </Scrollbars>
        <BottomBarContent sendMessage={sendMessage} />
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
      <VielCommLoadingButton
        variant="contained"
        loading={isLoading}
        onClick={connect}
        disabled={!clientId.length}
        loadingIndicator={<CircularProgress color="info" />}
      >
        {isLoading ? "" : "Connect"}
      </VielCommLoadingButton>
    </Stack>
  );
}

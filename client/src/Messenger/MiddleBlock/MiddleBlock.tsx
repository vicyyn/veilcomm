import {
  Box,
  Card,
  styled,
  Stack,
  TextField,
  useTheme,
  CircularProgress,
  Divider,
  Button,
  Typography,
} from "@mui/material";
import TopBarContent from "./TopBarContent";
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

export default function MiddleBlock(props: {
  setInitializing: (...args: any) => any;
  userKey: string | null;
}) {
  const theme = useTheme();
  const [connected, setConnected] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [clientId, setClientId] = useState("");
  const [conversation, setConversation] = useState<ReactNode[]>([]);
  const [clientType, setClientType] = useState(false);

  useEffect(() => {
    listen<string>("tor-change-receive-message", (event) =>
      receiveMessage(event.payload)
    );
  }, []);

  const generate = (n: number): string => {
    var add = 1,
      max = 12 - add;

    if (n > max) {
      return generate(max) + generate(n - max);
    }

    max = Math.pow(10, n + add);
    var min = max / 10;
    var number = Math.floor(Math.random() * (max - min + 1)) + min;

    return ("" + number).substring(add);
  };

  const VielCommLoadingButton = styled(LoadingButton)(({ theme }) => ({
    "&.Mui-disabled": {
      color: "grey",
    },
  }));

  const connect = () => {
    setIsLoading(true);
    emit("tor-event", "connect-" + clientType.toString());

    listen("tor-change-connected", () => {
      setIsLoading(false);
      setConnected(true);
    });
  };

  const sendMessage = (text: string) => {
    setConversation((prev) => [...prev, renderRight(text)]);
    emit("tor-event", "send-message");
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

  const init = () => {
    emit("tor-event", "initialize-" + clientType.toString());
    props.setInitializing(true);
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

  if (props.userKey && !connected) {
    return (
      <Stack
        m={1}
        gap={2}
        sx={{
          display: "flex",
          height: "100vh",
          justifyContent: "center",
          alignItems: "center",
        }}
      >
        Waiting for users to connect to you
      </Stack>
    );
  }

  return (
    <Stack
      m={1}
      gap={2}
      sx={{
        display: "flex",
        height: "100vh",
        justifyContent: "center",
        alignItems: "center",
      }}
    >
      <Stack gap={3} mt={-8}>
        <Stack
          direction={"row"}
          display={"flex"}
          justifyContent={"center"}
          alignItems={"center"}
          gap={2}
        >
          <label className="switch">
            <input
              type="checkbox"
              onChange={() => setClientType((prev) => !prev)}
            />
            <span className="slider"></span>
          </label>
          <Typography mt={2.5}>{clientType ? "user1" : "user2"}</Typography>
        </Stack>
        <Button variant="contained" onClick={init}>
          Initialize
        </Button>
      </Stack>
      OR
      <Stack direction={"row"} width={"80%"} ml={8}>
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
    </Stack>
  );
}

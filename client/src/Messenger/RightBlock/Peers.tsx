import {
  Stack,
  Box,
  Typography,
  Avatar,
  useTheme,
  Button,
  TableContainer,
  Table,
  TableHead,
  TableRow,
  TableCell,
  TableBody,
  Tooltip,
} from "@mui/material";
import RefreshIcon from "@mui/icons-material/Refresh";
import { useEffect, useState } from "react";
import { emit, listen } from "@tauri-apps/api/event";
import { Scrollbars } from "react-custom-scrollbars-2";

type Peer = {
  ip: string;
  port: number;
  id_key: string;
};

export default function Peers(props: { userKey: string | null }) {
  const theme = useTheme();
  const [peers, setPeers] = useState<Peer[]>([]);

  useEffect(() => {
    listen<Peer[]>("tor-change-fetch-relays", (event) => {
      setPeers(event.payload);
    });
  }, []);

  useEffect(() => {
    fetchRelays();
  }, [props.userKey]);

  const fetchRelays = () => {
    emit("tor-event", "fetch-relays");
  };

  return (
    <>
      <Box
        padding={2.45}
        sx={{ borderBottom: `${theme.colors.alpha.black[50]} solid 2px` }}
      >
        <Stack
          direction={"row"}
          gap={1}
          display={"flex"}
          alignItems={"center"}
          justifyContent={"space-between"}
        >
          <Stack direction={"row"} gap={1}>
            <Typography>Address: </Typography>
            <Box display={"flex"} alignItems={"center"}>
              <Typography
                maxWidth={'400px'}
                variant="body1"
                style={{ overflowWrap: "break-word" }}
              >
                {props.userKey}
              </Typography>
            </Box>
          </Stack>
          <Tooltip title="fetch relays" arrow>
            <Button variant="contained" onClick={fetchRelays}>
              <RefreshIcon />
            </Button>
          </Tooltip>
        </Stack>
      </Box>

      <Box p={2} height={"100vh"}>
        <Scrollbars>
          <Stack>
            <Typography variant="h3" my={1} color={"white"}>
              Relays
            </Typography>
            <TableContainer>
              <Table sx={{ border: "white solid 1px" }}>
                <TableHead>
                  <TableRow>
                    <TableCell>IP</TableCell>
                    <TableCell>Port</TableCell>
                    <TableCell>Id Key</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {peers.map((peer) => (
                    <TableRow
                      hover
                      key={peer.id_key}
                      sx={{ "&:last-child td, &:last-child th": { border: 0 } }}
                    >
                      <TableCell component="th" scope="row">
                        {peer.ip}
                      </TableCell>
                      <TableCell>{peer.port}</TableCell>
                      <TableCell>
                        <Typography>{peer.id_key.slice(0, 32)}...</Typography>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
          </Stack>
        </Scrollbars>
      </Box>
    </>
  );
}

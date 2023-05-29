import { Box, Avatar, Typography, Tooltip } from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";

function TopBarContent(props: {
  clientId: string;
  endConversation: () => any;
}) {
  return (
    <Box display="flex" alignItems="center" justifyContent={"space-between"}>
      <Box>
        <Avatar
          sx={{
            width: 48,
            height: 48,
          }}
        />
        <Box ml={1}>
          <Typography>{props.clientId}</Typography>
        </Box>
      </Box>
      <Tooltip title={"End conversation"} arrow>
        <CloseIcon
          color="error"
          onClick={props.endConversation}
          sx={{ cursor: "pointer" }}
        />
      </Tooltip>
    </Box>
  );
}

export default TopBarContent;
